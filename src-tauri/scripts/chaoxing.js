/**
 * 绍理闪连 - 超星学习通自动化脚本
 *
 * 注入到 Tauri WebView 中，实现视频自动播放和章节测试自动答题。
 * 通过 window.__TAURI__.core.invoke() 与 Rust 后端通信。
 * 不使用任何第三方脚本或 Tampermonkey API。
 */
(function () {
  'use strict';

  // ── 拦截 window.open：超星课程页面大量用 window.open 导航，
  //    WebView2 默认会打开新窗口或拦截，我们强制在当前窗口跳转 ──
  const _origOpen = window.open;
  window.open = function (url, name, features) {
    if (url && url !== 'about:blank' && url !== '') {
      window.location.href = url;
      return window;
    }
    return _origOpen.call(this, url, name, features);
  };

  // ── 域名守卫 ──
  const hostname = window.location.hostname;
  if (
    !hostname.includes('chaoxing.com') &&
    !hostname.includes('edu.cn') &&
    !hostname.includes('org.cn')
  ) {
    return;
  }

  // ── 防止重复注入 ──
  if (window.__COURSE_HELPER__) {
    return;
  }

  // 拦截 <a target="_blank"> 点击，在当前窗口跳转（超星课程页面使用此方式导航）
  document.addEventListener("click", function(e) {
    var a = e.target.closest("a");
    if (a && a.href && a.getAttribute("target") === "_blank") {
      e.preventDefault();
      e.stopPropagation();
      window.location.href = a.href;
    }
  }, true);

  // ── DOM 选择器常量 ──
  const SELECTORS = {
    VIDEO_CONTAINER: '#video, #audio',
    MEDIA_ELEMENT: 'video, audio',
    ERROR_DIALOG: '.vjs-modal-dialog-content',
    QUIZ_CONTAINER: '.TiMu',
    QUIZ_TITLE: '.Zy_TItle .clearfix, .Zy_TItle',
    QUIZ_OPTIONS: 'ul li',
    QUIZ_OPTION_LABEL: 'ul li .after, ul li label:not(.before)',
    ANSWER_TYPE_INPUT: 'input[id^="answertype"]',
    SUBMIT_BTN: '#submitBtn, .Btn_blue_1, .submitBtn',
    CHAPTER_LIST: '[onclick^="getTeacherAjax"]',
    UNFINISHED_BADGE: '.jobUnfinishCount',
    COMPLETED_ICON: '.icon_Completed',
  };

  // 题型映射（与超星 answertype 值一致）
  const QUESTION_TYPE = {
    0: 'single',    // 单选题
    1: 'multiple',  // 多选题
    3: 'judgement', // 判断题
  };

  // ── 状态管理 ──
  const STATE = {
    running: false,
    speed: 2.0,
    currentChapter: '',
    videosCompleted: 0,
    quizzesAnswered: 0,
    quizzesMissed: 0,
    log: [],
  };

  // ── 工具函数 ──
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

  function addLog(msg) {
    const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });
    const entry = `[${time}] ${msg}`;
    STATE.log.push(entry);
    if (STATE.log.length > 200) STATE.log.shift();
    console.log('[CourseHelper]', entry);
  }

  async function invokeSafe(command, args) {
    try {
      if (window.__TAURI__ && window.__TAURI__.core) {
        return await window.__TAURI__.core.invoke(command, args);
      }
    } catch (e) {
      console.warn('[CourseHelper] invoke error:', command, e);
    }
    return null;
  }

  // ── 查找 iframe 中的文档 ──
  function findIframeDocuments() {
    const docs = [document];
    try {
      const iframes = document.querySelectorAll('iframe');
      iframes.forEach((iframe) => {
        try {
          if (iframe.contentDocument) {
            docs.push(iframe.contentDocument);
            // 超星可能嵌套两层 iframe
            const nested = iframe.contentDocument.querySelectorAll('iframe');
            nested.forEach((n) => {
              try {
                if (n.contentDocument) docs.push(n.contentDocument);
              } catch (_) {}
            });
          }
        } catch (_) {
          // 跨域 iframe 忽略
        }
      });
    } catch (_) {}
    return docs;
  }

  // ── 等待媒体元素出现 ──
  async function waitForMedia(timeout) {
    timeout = timeout || 15000;
    const start = Date.now();

    while (Date.now() - start < timeout) {
      const docs = findIframeDocuments();
      for (const doc of docs) {
        // 先检查视频容器是否存在
        const container = doc.querySelector(SELECTORS.VIDEO_CONTAINER);
        if (!container) continue;

        // 查找实际的 media 元素
        const media = doc.querySelector(SELECTORS.MEDIA_ELEMENT);
        if (media && media.src && media.src.length > 0) {
          return media;
        }
      }
      await sleep(1000);
    }
    return null;
  }

  // ── 检测错误弹窗 ──
  function checkVideoError() {
    const docs = findIframeDocuments();
    for (const doc of docs) {
      const errorEl = doc.querySelector(SELECTORS.ERROR_DIALOG);
      if (errorEl && errorEl.textContent.trim()) {
        return errorEl.textContent.trim();
      }
    }
    return null;
  }

  // ── 视频播放核心 ──
  async function playVideo(media) {
    addLog('开始播放视频');

    try {
      media.muted = true; // 静音避免干扰
      media.currentTime = 0;
      media.playbackRate = STATE.speed;
      await media.play();
    } catch (e) {
      addLog('播放失败: ' + e.message);
      return false;
    }

    return new Promise((resolve) => {
      let resolved = false;

      function done(result) {
        if (resolved) return;
        resolved = true;
        media.removeEventListener('ended', onEnded);
        media.removeEventListener('pause', onPause);
        resolve(result);
      }

      function onEnded() {
        addLog('视频播放完成');
        STATE.videosCompleted++;
        done(true);
      }

      function onPause() {
        if (!media.ended && STATE.running) {
          // 短暂延迟后恢复播放（避免与正常操作冲突）
          setTimeout(() => {
            if (STATE.running && !media.ended) {
              media.play().catch(() => {});
            }
          }, 1500);
        }
      }

      media.addEventListener('ended', onEnded, { once: true });
      media.addEventListener('pause', onPause);

      // 定期检查错误弹窗
      const errorCheck = setInterval(() => {
        const error = checkVideoError();
        if (error) {
          addLog('检测到视频错误: ' + error);
          clearInterval(errorCheck);
          done(false);
        }
        if (!STATE.running) {
          clearInterval(errorCheck);
          done(false);
        }
      }, 3000);

      // 当 done 被调用时清理 interval
      const origDone = done;
      done = function (r) {
        clearInterval(errorCheck);
        origDone(r);
      };
    });
  }

  // ── 检测题目类型 ──
  function detectQuestionType(questionEl) {
    const typeInput = questionEl.querySelector(SELECTORS.ANSWER_TYPE_INPUT);
    if (typeInput) {
      const val = parseInt(typeInput.value, 10);
      return QUESTION_TYPE[val] || 'unknown';
    }
    return 'unknown';
  }

  // ── 提取题目文本 ──
  function extractQuestionText(questionEl) {
    const titleEl = questionEl.querySelector(SELECTORS.QUIZ_TITLE);
    if (titleEl) {
      return titleEl.textContent.trim();
    }
    // 降级：取整个容器的文本
    return questionEl.textContent.trim().substring(0, 500);
  }

  // ── 自动答题 ──
  async function handleQuiz() {
    const docs = findIframeDocuments();
    let quizFound = false;

    for (const doc of docs) {
      const questions = doc.querySelectorAll(SELECTORS.QUIZ_CONTAINER);
      if (questions.length === 0) continue;

      quizFound = true;
      addLog('检测到 ' + questions.length + ' 道题目');

      for (const qEl of questions) {
        const questionText = extractQuestionText(qEl);
        const questionType = detectQuestionType(qEl);

        if (!questionText) continue;

        // 通过 IPC 查题
        const result = await invokeSafe('course_match_question', {
          question: questionText,
        });

        if (result) {
          addLog('匹配到答案，题型: ' + questionType);
          await selectAnswer(qEl, result, questionType);
          STATE.quizzesAnswered++;
        } else {
          addLog('未匹配到答案，跳过');
          STATE.quizzesMissed++;
        }

        await sleep(500); // 每题间隔
      }

      // 尝试提交
      await sleep(1000);
      const submitBtn = doc.querySelector(SELECTORS.SUBMIT_BTN);
      if (submitBtn) {
        submitBtn.click();
        addLog('已点击提交按钮');
      }
    }

    return quizFound;
  }

  // ── 选择答案 ──
  async function selectAnswer(questionEl, answer, type) {
    const options = questionEl.querySelectorAll(SELECTORS.QUIZ_OPTIONS);
    if (options.length === 0) return;

    const answerLower = answer.toLowerCase().trim();

    if (type === 'single' || type === 'judgement') {
      // 单选/判断：选最匹配的选项
      let bestMatch = null;
      let bestScore = 0;

      options.forEach((opt) => {
        const optText = opt.textContent.trim().toLowerCase();
        // 简单的包含匹配 + 相似度
        const score = calculateSimpleMatch(answerLower, optText);
        if (score > bestScore) {
          bestScore = score;
          bestMatch = opt;
        }
      });

      if (bestMatch) {
        const clickable =
          bestMatch.querySelector('input') ||
          bestMatch.querySelector('label') ||
          bestMatch;
        clickable.click();
      }
    } else if (type === 'multiple') {
      // 多选：答案可能是 "AB" 或 "A,B" 或完整文本
      const answerParts = answerLower
        .replace(/[,，、\s]+/g, '')
        .split('');

      options.forEach((opt, idx) => {
        const optLetter = String.fromCharCode(97 + idx); // a, b, c, d...
        const optText = opt.textContent.trim().toLowerCase();

        let shouldSelect = false;
        // 检查字母匹配
        if (answerParts.includes(optLetter)) {
          shouldSelect = true;
        }
        // 检查文本匹配
        if (answerLower.includes(optText) || optText.includes(answerLower)) {
          shouldSelect = true;
        }

        if (shouldSelect) {
          const clickable =
            opt.querySelector('input') ||
            opt.querySelector('label') ||
            opt;
          clickable.click();
        }
      });
    }
  }

  // ── 简单文本匹配分数 ──
  function calculateSimpleMatch(answer, option) {
    if (!answer || !option) return 0;

    // 完全包含
    if (option.includes(answer) || answer.includes(option)) return 0.9;

    // 字符重叠率
    const answerChars = new Set(answer.split(''));
    const optionChars = new Set(option.split(''));
    let overlap = 0;
    answerChars.forEach((c) => {
      if (optionChars.has(c)) overlap++;
    });
    return overlap / Math.max(answerChars.size, optionChars.size);
  }

  // ── 章节切换 ──
  async function nextChapter() {
    // 方式1：尝试调用超星内置翻页 API
    try {
      if (typeof window.PCount !== 'undefined' && window.PCount.next) {
        window.PCount.next();
        addLog('调用 PCount.next() 切换下一节');
        await sleep(3000);
        return true;
      }
    } catch (_) {}

    // 方式2：点击下一个未完成章节
    try {
      const chapters = document.querySelectorAll(SELECTORS.CHAPTER_LIST);
      for (const chapter of chapters) {
        // 检查是否有未完成标记
        const parent = chapter.closest('li, .chapter-item, .ncells');
        if (parent) {
          const completed = parent.querySelector(SELECTORS.COMPLETED_ICON);
          if (!completed) {
            chapter.click();
            addLog('点击下一个未完成章节');
            await sleep(3000);
            return true;
          }
        }
      }
    } catch (_) {}

    addLog('未找到下一节');
    return false;
  }

  // ── 上报进度 ──
  async function reportProgress() {
    await invokeSafe('course_report_progress', {
      videos: STATE.videosCompleted,
      quizzes: STATE.quizzesAnswered,
      chapter: STATE.currentChapter,
      status: STATE.running ? 'running' : 'stopped',
    });
  }

  // ── 主循环 ──
  async function mainLoop() {
    addLog('主循环启动');

    while (STATE.running) {
      // 获取当前章节名
      try {
        const chapterEl = document.querySelector('#curChapterId');
        if (chapterEl) {
          STATE.currentChapter = chapterEl.textContent?.trim() || chapterEl.value || '';
        }
      } catch (_) {}

      // 1. 尝试查找并播放视频
      const media = await waitForMedia(15000);
      if (media && STATE.running) {
        await playVideo(media);
        await reportProgress();

        if (!STATE.running) break;
        await sleep(2000);
      }

      // 2. 检测章节测试
      const hasQuiz = await handleQuiz();
      if (hasQuiz) {
        await reportProgress();
        if (!STATE.running) break;
        await sleep(2000);
      }

      // 3. 如果没有视频也没有测试，尝试下一节
      if (!media && !hasQuiz) {
        const hasNext = await nextChapter();
        if (!hasNext) {
          addLog('所有章节已完成');
          STATE.running = false;
          STATE.status = 'completed';
          await reportProgress();
          break;
        }
      } else if (media) {
        // 视频播完了，尝试下一节
        await sleep(1000);
        await nextChapter();
      }

      // 每轮间隔
      await sleep(2000);
    }

    addLog('主循环结束');
  }

  // ── 公开接口 ──
  window.__COURSE_HELPER__ = {
    start: function () {
      if (STATE.running) return;
      STATE.running = true;
      addLog('刷课助手已启动');
      mainLoop();
    },

    stop: function () {
      STATE.running = false;
      addLog('刷课助手已停止');
      reportProgress();
    },

    setSpeed: function (speed) {
      STATE.speed = parseFloat(speed) || 2.0;
      addLog('倍速设置为 ' + STATE.speed + 'x');
    },

    getStatus: function () {
      return {
        running: STATE.running,
        speed: STATE.speed,
        videosCompleted: STATE.videosCompleted,
        quizzesAnswered: STATE.quizzesAnswered,
        quizzesMissed: STATE.quizzesMissed,
        currentChapter: STATE.currentChapter,
        log: STATE.log.slice(-50),
      };
    },
  };

  addLog('脚本已加载，等待启动指令');
})();

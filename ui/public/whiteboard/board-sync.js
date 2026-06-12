/**
 * board-sync.js
 * 白板数据同步模块
 * 
 * 功能：
 * 1. 在白板窗口打开时，从 Tauri 加载数据并写入 localforage
 * 2. 定期从 localforage 读取数据并同步到 Tauri
 * 3. 在白板窗口关闭时，最后同步一次
 */

(function() {
  'use strict';
  
  const BOARD_NAME = window.__BOARD_NAME__ || 'default';
  const STORAGE_KEY = 'main_board_content';
  const SYNC_INTERVAL = 5000; // 5 秒同步一次
  
  let syncTimer = null;
  let lastData = null;
  
  // 等待 localforage 可用
  function waitForLocalforage() {
    return new Promise((resolve) => {
      if (window.localforage) {
        resolve(window.localforage);
        return;
      }
      
      // 检查 localforage 是否已经通过模块加载
      const checkInterval = setInterval(() => {
        if (window.localforage) {
          clearInterval(checkInterval);
          resolve(window.localforage);
        }
      }, 100);
      
      // 超时处理
      setTimeout(() => {
        clearInterval(checkInterval);
        resolve(null);
      }, 10000);
    });
  }
  
  // 从 Tauri 加载白板数据
  async function loadFromTauri() {
    try {
      if (window.__TAURI__ && window.__TAURI__.core) {
        const data = await window.__TAURI__.core.invoke('load_whiteboard_data', {
          name: BOARD_NAME
        });
        console.log(`[BoardSync] 从 Tauri 加载白板数据: ${BOARD_NAME}`);
        return data;
      }
    } catch (e) {
      console.warn('[BoardSync] 从 Tauri 加载失败:', e);
    }
    return null;
  }
  
  // 保存数据到 Tauri
  async function saveToTauri(data) {
    try {
      if (window.__TAURI__ && window.__TAURI__.core) {
        await window.__TAURI__.core.invoke('save_whiteboard_data', {
          name: BOARD_NAME,
          data: data
        });
        console.log(`[BoardSync] 同步白板数据到 Tauri: ${BOARD_NAME}`);
        return true;
      }
    } catch (e) {
      console.warn('[BoardSync] 同步到 Tauri 失败:', e);
    }
    return false;
  }
  
  // 从 localforage 读取数据
  async function readFromLocalforage(lf) {
    try {
      const data = await lf.getItem(STORAGE_KEY);
      return data;
    } catch (e) {
      console.warn('[BoardSync] 从 localforage 读取失败:', e);
      return null;
    }
  }
  
  // 写入数据到 localforage
  async function writeToLocalforage(lf, data) {
    try {
      await lf.setItem(STORAGE_KEY, data);
      console.log('[BoardSync] 写入 localforage 成功');
      return true;
    } catch (e) {
      console.warn('[BoardSync] 写入 localforage 失败:', e);
      return false;
    }
  }
  
  // 检查数据是否发生变化
  function hasDataChanged(newData) {
    if (!lastData || !newData) return true;
    return JSON.stringify(lastData) !== JSON.stringify(newData);
  }
  
  // 同步数据
  async function syncData(lf) {
    const currentData = await readFromLocalforage(lf);
    
    if (currentData && hasDataChanged(currentData)) {
      await saveToTauri(currentData);
      lastData = currentData;
    }
  }
  
  // 开始定期同步
  function startPeriodicSync(lf) {
    if (syncTimer) clearInterval(syncTimer);
    
    syncTimer = setInterval(() => {
      syncData(lf);
    }, SYNC_INTERVAL);
    
    console.log(`[BoardSync] 开始定期同步，间隔 ${SYNC_INTERVAL}ms`);
  }
  
  // 停止定期同步
  function stopPeriodicSync() {
    if (syncTimer) {
      clearInterval(syncTimer);
      syncTimer = null;
      console.log('[BoardSync] 停止定期同步');
    }
  }
  
  // 初始化
  async function init() {
    console.log(`[BoardSync] 初始化白板数据同步: ${BOARD_NAME}`);
    
    // 等待 localforage 可用
    const lf = await waitForLocalforage();
    if (!lf) {
      console.error('[BoardSync] localforage 不可用');
      return;
    }
    
    // 从 Tauri 加载数据
    const tauriData = await loadFromTauri();
    if (tauriData) {
      // 写入 localforage
      await writeToLocalforage(lf, tauriData);
      lastData = tauriData;
      console.log('[BoardSync] 已从 Tauri 加载数据到 localforage');
    } else {
      // 从 localforage 读取现有数据
      const existingData = await readFromLocalforage(lf);
      if (existingData) {
        lastData = existingData;
        console.log('[BoardSync] 使用 localforage 中的现有数据');
      } else {
        console.log('[BoardSync] 没有现有数据，将创建新白板');
      }
    }
    
    // 开始定期同步
    startPeriodicSync(lf);
    
    // 监听窗口关闭事件
    window.addEventListener('beforeunload', async () => {
      stopPeriodicSync();
      // 最后同步一次
      await syncData(lf);
    });
    
    // 监听 Tauri 事件
    if (window.__TAURI__ && window.__TAURI__.event) {
      // 监听来自主窗口的保存请求
      window.__TAURI__.event.listen('request-save', async () => {
        await syncData(lf);
      });
    }
    
    console.log('[BoardSync] 初始化完成');
  }
  
  // 导出到全局，供其他脚本使用
  window.__BOARD_SYNC__ = {
    sync: async () => {
      const lf = window.localforage;
      if (lf) await syncData(lf);
    },
    load: loadFromTauri,
    save: saveToTauri,
    stop: stopPeriodicSync
  };
  
  // 启动初始化
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();

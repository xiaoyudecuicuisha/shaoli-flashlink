//! 清理规则解析器：JSON 规则加载、环境变量展开、内置规则

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanRule {
    pub name: String,
    pub path: String,
    pub rule_type: String,
    pub default_enabled: bool,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateItem {
    pub name: String,
    pub category: String,
    pub estimated_bytes: u64,
    pub file_count: u64,
}

/// Expand environment variables in a path string
pub fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    let env_vars: HashMap<String, String> =
        std::env::vars().collect();

    let mut changed = true;
    while changed {
        changed = false;
        if let Some(start) = result.find('%') {
            if let Some(end) = result[start + 1..].find('%') {
                let var_name = &result[start + 1..start + 1 + end];
                if let Some(value) = env_vars.get(var_name) {
                    result = format!(
                        "{}{}{}",
                        &result[..start],
                        value,
                        &result[start + 2 + end..]
                    );
                    changed = true;
                }
            }
        }
    }

    result
}

/// 解析 JSON 数组格式的规则条目 [name, path, type, default, desc, enabled]
pub fn parse_rule_entry(entry: &serde_json::Value) -> Option<CleanRule> {
    let arr = entry.as_array()?;
    if arr.len() < 6 {
        return None;
    }

    let name = arr[0].as_str()?.to_string();
    let raw_path = arr[1].as_str()?.to_string();
    let rule_type = arr[2].as_str()?.to_string();
    let default_enabled = arr[3].as_bool().unwrap_or(false);
    let description = arr[4].as_str().unwrap_or("").to_string();

    let category = name
        .split(" | ")
        .next()
        .unwrap_or("其他")
        .to_string();

    let path = expand_env_vars(&raw_path);

    Some(CleanRule {
        name,
        path,
        rule_type,
        default_enabled,
        category,
        description,
    })
}

/// Load rules from embedded JSON resource
pub fn load_rules_from_json(json_str: &str) -> Vec<CleanRule> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return Vec::new();
    };

    let Some(arr) = value.as_array() else {
        return Vec::new();
    };

    arr.iter().filter_map(parse_rule_entry).collect()
}

/// Load all bundled rule files
pub fn load_all_rules() -> Vec<CleanRule> {
    let mut rules = Vec::new();

    let rule_files = [
        include_str!("../../resources/rules/bleachbit.json"),
        include_str!("../../resources/rules/winapp2.json"),
        include_str!("../../resources/rules/rules_cn_apps.json"),
        include_str!("../../resources/rules/rules_dev_tools.json"),
        include_str!("../../resources/rules/rules_ai_tools.json"),
    ];

    for json_str in &rule_files {
        rules.extend(load_rules_from_json(json_str));
    }

    rules
}

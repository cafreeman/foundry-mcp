//! Parse `- [ ]` / `- [x]` lines from `task-list.md`.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TaskItem {
    pub done: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TaskListSummary {
    pub total: usize,
    pub complete: usize,
    pub items: Vec<TaskItem>,
}

pub fn parse_task_list(markdown: &str) -> TaskListSummary {
    let mut items = Vec::new();
    for line in markdown.lines() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("- [") else {
            continue;
        };
        let Some((mark, after)) = rest.split_once(']') else {
            continue;
        };
        let done = matches!(mark, "x" | "X");
        let desc = after.trim_start().trim_start_matches('-').trim();
        items.push(TaskItem {
            done,
            description: desc.to_string(),
        });
    }
    let total = items.len();
    let complete = items.iter().filter(|i| i.done).count();
    TaskListSummary {
        total,
        complete,
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_checkboxes() {
        let md = "- [ ] one\n- [x] two\n- [X] three\n";
        let s = parse_task_list(md);
        assert_eq!(s.total, 3);
        assert_eq!(s.complete, 2);
        assert!(!s.items[0].done);
        assert!(s.items[1].done);
    }
}

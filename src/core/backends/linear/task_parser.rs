use comrak::{
    Arena, ComrakOptions,
    nodes::{AstNode, NodeValue},
    parse_document,
};

/// Parse a markdown checklist into DesiredTask entries using comrak (GFM task lists).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesiredTask {
    pub text: String,
    pub completed: bool,
}

pub fn parse_task_list(markdown: &str) -> Vec<DesiredTask> {
    let mut opts = ComrakOptions::default();
    opts.extension.tasklist = true; // enable GFM task list parsing

    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &opts);

    let mut tasks = Vec::new();
    fn walk<'a>(node: &'a AstNode<'a>, tasks: &mut Vec<DesiredTask>) {
        // If this node is a task item, collect its text and completion state
        if let NodeValue::TaskItem(checked) = &node.data.borrow().value {
            let text = collect_text(node).trim().to_string();
            let completed = checked.map(|c| c != ' ' && c != '\0').unwrap_or(false);
            tasks.push(DesiredTask { text, completed });
        }
        for c in node.children() {
            walk(c, tasks);
        }
    }

    fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
        let mut out = String::new();
        fn gather<'a>(n: &'a AstNode<'a>, out: &mut String) {
            match &n.data.borrow().value {
                NodeValue::Text(lit) => {
                    out.push_str(lit);
                    out.push(' ');
                }
                NodeValue::Code(code) => {
                    out.push_str(&code.literal);
                    out.push(' ');
                }
                NodeValue::LineBreak | NodeValue::SoftBreak => {
                    out.push(' ');
                }
                _ => {}
            }
            for c in n.children() {
                gather(c, out);
            }
        }
        gather(node, &mut out);
        out
    }

    walk(root, &mut tasks);
    tasks
}

#[cfg(test)]
mod tests {
    use super::{DesiredTask, parse_task_list};

    #[test]
    fn parses_basic_checklist() {
        let md = "## Phase D: Tasks\n- [ ] Implement reconciliation\n- [x] Ensure idempotency\n";
        let tasks = parse_task_list(md);
        assert_eq!(tasks.len(), 2);
        assert_eq!(
            tasks[0],
            DesiredTask {
                text: "Implement reconciliation".to_string(),
                completed: false
            }
        );
        assert_eq!(
            tasks[1],
            DesiredTask {
                text: "Ensure idempotency".to_string(),
                completed: true
            }
        );
    }

    #[test]
    fn ignores_non_checklist_lines_and_headers() {
        let md = "# Title\nSome description\n* [ ] Item A\n- Not a checkbox\n* [X] Item B\n";
        let tasks = parse_task_list(md);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].text, "Item A");
        assert!(!tasks[0].completed);
        assert_eq!(tasks[1].text, "Item B");
        assert!(tasks[1].completed);
    }
}

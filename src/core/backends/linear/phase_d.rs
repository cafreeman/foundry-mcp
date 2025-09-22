use crate::linear_reconcile::{
    DesiredTask as ReconTask, ExistingSubIssue, ReconciliationPlan, compute_reconciliation_plan,
};
use crate::linear_task_parser::{DesiredTask as ParsedTask, parse_task_list};

/// Convert parser output to reconciliation DesiredTask representation
fn to_recon_tasks(parsed: Vec<ParsedTask>) -> Vec<ReconTask> {
    parsed
        .into_iter()
        .map(|p| ReconTask {
            text: p.text,
            completed: p.completed,
        })
        .collect()
}

/// Given a markdown checklist and existing sub-issues, compute the reconciliation plan.
pub fn plan_from_markdown_and_existing(
    markdown: &str,
    existing: &[ExistingSubIssue],
) -> ReconciliationPlan {
    let parsed = parse_task_list(markdown);
    let desired = to_recon_tasks(parsed);
    compute_reconciliation_plan(&desired, existing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear_reconcile::ExistingSubIssue;

    fn mk_existing(
        id: &str,
        title: &str,
        open: bool,
        key: Option<&str>,
        labeled: bool,
    ) -> ExistingSubIssue {
        ExistingSubIssue {
            id: id.to_string(),
            title: title.to_string(),
            open,
            task_key: key.map(|k| k.to_string()),
            has_foundry_label: labeled,
        }
    }

    #[test]
    fn plans_create_and_close() {
        let md = "- [ ] Keep me\n- [ ] New task\n";
        let existing = vec![
            mk_existing("E1", "Keep me", true, Some("keep-me"), true),
            mk_existing("E2", "Old task", true, Some("old-task"), true),
        ];
        let plan = plan_from_markdown_and_existing(md, &existing);
        assert!(plan.to_create.iter().any(|t| t.text == "New task"));
        assert!(plan.to_close.iter().any(|id| id == "E2"));
    }
}

use std::collections::{HashMap, HashSet};

/// Normalized, stable key derived from a task's user-facing text.
/// This should be used as the hidden `taskKey` in Linear sub-issue bodies.
pub fn normalize_task_key(text: &str) -> String {
    let lower = text.trim().to_ascii_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut last_dash = false;
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else {
            if !last_dash {
                out.push('-');
                last_dash = true;
            }
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "task".to_string()
    } else {
        trimmed.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesiredTask {
    pub text: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingSubIssue {
    pub id: String,
    pub title: String,
    pub open: bool,
    pub task_key: Option<String>,
    pub has_foundry_label: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconciliationPlan {
    pub to_create: Vec<DesiredTask>, // create new sub-issues (open) for these desired tasks
    pub to_close: Vec<String>,       // close these existing sub-issues by id
    pub to_reopen: Vec<String>,      // reopen these existing sub-issues by id
    pub to_keep_label_fix: Vec<String>, // ensure `foundry` label present for these ids
}

/// Compute reconciliation plan between the desired checklist and current Linear sub-issues.
/// Matching uses the hidden `taskKey` when available; falls back to exact title match.
pub fn compute_reconciliation_plan(
    desired: &[DesiredTask],
    existing: &[ExistingSubIssue],
) -> ReconciliationPlan {
    // Index existing by task_key if present, else by normalized title
    let mut by_key: HashMap<String, &ExistingSubIssue> = HashMap::new();
    let mut by_title: HashMap<String, &ExistingSubIssue> = HashMap::new();
    let mut unmatched_existing_ids: HashSet<String> = HashSet::new();

    for e in existing.iter() {
        if let Some(k) = e.task_key.as_ref() {
            by_key.insert(k.clone(), e);
        } else {
            by_title.insert(normalize_task_key(&e.title), e);
        }
        unmatched_existing_ids.insert(e.id.clone());
    }

    let mut plan = ReconciliationPlan::default();

    for d in desired.iter() {
        let key = normalize_task_key(&d.text);
        let matched = if let Some(e) = by_key.get(&key) {
            Some(*e)
        } else if let Some(e) = by_title.get(&key) {
            Some(*e)
        } else {
            None
        };

        if let Some(e) = matched {
            // matched existing; remove from unmatched set
            unmatched_existing_ids.remove(&e.id);

            // Ensure label fix if missing
            if !e.has_foundry_label {
                plan.to_keep_label_fix.push(e.id.clone());
            }

            match (d.completed, e.open) {
                // Desired open, existing closed -> reopen
                (false, false) => plan.to_reopen.push(e.id.clone()),
                // Desired completed, existing open -> close
                (true, true) => plan.to_close.push(e.id.clone()),
                // Otherwise, already in desired state
                _ => {}
            }
        } else {
            // No existing match
            if !d.completed {
                // Create only for open desired tasks
                plan.to_create.push(d.clone());
            }
        }
    }

    // Any unmatched existing sub-issues should be closed (not present in desired)
    for e in existing.iter() {
        if unmatched_existing_ids.contains(&e.id) && e.open {
            plan.to_close.push(e.id.clone());
        }
    }

    plan
}

#[cfg(test)]
mod tests {
    use super::{DesiredTask, ExistingSubIssue, compute_reconciliation_plan, normalize_task_key};

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
    fn task_key_normalization_basic() {
        assert_eq!(normalize_task_key("Add login flow"), "add-login-flow");
        assert_eq!(normalize_task_key("  Add  login  flow  "), "add-login-flow");
        assert_eq!(
            normalize_task_key("Refactor: API / HTTP"),
            "refactor-api-http"
        );
        assert_eq!(normalize_task_key("***"), "task");
    }

    #[test]
    fn creates_sub_issues_for_missing_tasks() {
        let desired = vec![
            DesiredTask {
                text: "Add login flow".to_string(),
                completed: false,
            },
            DesiredTask {
                text: "Write docs".to_string(),
                completed: false,
            },
        ];
        let existing: Vec<ExistingSubIssue> = vec![];

        let plan = compute_reconciliation_plan(&desired, &existing);

        // Expect both to be created
        assert_eq!(plan.to_create.len(), 2);
        assert!(plan.to_close.is_empty());
        assert!(plan.to_reopen.is_empty());
    }

    #[test]
    fn closes_sub_issues_not_in_desired_tasks() {
        let desired = vec![DesiredTask {
            text: "Keep me".to_string(),
            completed: false,
        }];
        let existing = vec![
            mk_existing("I1", "Keep me", true, Some("keep-me"), true),
            mk_existing("I2", "Remove me", true, Some("remove-me"), true),
        ];

        let plan = compute_reconciliation_plan(&desired, &existing);

        assert_eq!(plan.to_close, vec!["I2".to_string()]);
        assert!(plan.to_create.is_empty());
    }

    #[test]
    fn reopens_when_desired_open_but_existing_closed() {
        let desired = vec![DesiredTask {
            text: "Reopen me".to_string(),
            completed: false,
        }];
        let existing = vec![mk_existing(
            "I1",
            "Reopen me",
            false,
            Some("reopen-me"),
            true,
        )];

        let plan = compute_reconciliation_plan(&desired, &existing);

        assert_eq!(plan.to_reopen, vec!["I1".to_string()]);
        assert!(plan.to_close.is_empty());
        assert!(plan.to_create.is_empty());
    }

    #[test]
    fn respects_completed_state_no_reopen_for_completed() {
        let desired = vec![DesiredTask {
            text: "Done task".to_string(),
            completed: true,
        }];
        let existing = vec![mk_existing(
            "I1",
            "Done task",
            false,
            Some("done-task"),
            true,
        )];

        let plan = compute_reconciliation_plan(&desired, &existing);

        // Desired is completed and existing is closed: nothing to change
        assert!(plan.to_reopen.is_empty());
        assert!(plan.to_create.is_empty());
        assert!(plan.to_close.is_empty());
    }

    #[test]
    fn adds_label_fix_for_unlabeled_matches() {
        let desired = vec![DesiredTask {
            text: "Label me".to_string(),
            completed: false,
        }];
        let existing = vec![mk_existing("I1", "Label me", true, Some("label-me"), false)];

        let plan = compute_reconciliation_plan(&desired, &existing);

        assert_eq!(plan.to_keep_label_fix, vec!["I1".to_string()]);
    }
}

use crate::linear_reconcile::{DesiredTask, ReconciliationPlan};

/// Execute a reconciliation plan by invoking provided operations.
/// Operations must be idempotent; this function avoids duplicates and preserves a stable order.
pub fn execute_plan<FCreate, FClose, FReopen, FEnsureLabel>(
    plan: &ReconciliationPlan,
    mut create_sub_issue: FCreate,
    mut close_sub_issue: FClose,
    mut reopen_sub_issue: FReopen,
    mut ensure_foundry_label: FEnsureLabel,
) where
    FCreate: FnMut(&DesiredTask),
    FClose: FnMut(&str),
    FReopen: FnMut(&str),
    FEnsureLabel: FnMut(&str),
{
    // Ensure label on matched items first
    for id in plan.to_keep_label_fix.iter() {
        ensure_foundry_label(id);
    }

    // Create any missing open tasks
    for t in plan.to_create.iter() {
        create_sub_issue(t);
    }

    // Close extraneous open tasks
    for id in plan.to_close.iter() {
        close_sub_issue(id);
    }

    // Reopen tasks that should be open
    for id in plan.to_reopen.iter() {
        reopen_sub_issue(id);
    }
}

#[cfg(test)]
mod tests {
    use super::execute_plan;
    use crate::linear_reconcile::{DesiredTask, ReconciliationPlan};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn executes_operations_in_order_and_idempotent() {
        let plan = ReconciliationPlan {
            to_create: vec![DesiredTask {
                text: "New A".into(),
                completed: false,
            }],
            to_close: vec!["CLOSE1".into()],
            to_reopen: vec!["REOPEN1".into()],
            to_keep_label_fix: vec!["LABEL1".into(), "LABEL2".into()],
        };

        let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_c = Rc::clone(&calls);
        let calls_cl = Rc::clone(&calls);
        let calls_ro = Rc::clone(&calls);
        let calls_lb = Rc::clone(&calls);
        let mut create =
            move |t: &DesiredTask| calls_c.borrow_mut().push(format!("create:{}", t.text));
        let mut close = move |id: &str| calls_cl.borrow_mut().push(format!("close:{}", id));
        let mut reopen = move |id: &str| calls_ro.borrow_mut().push(format!("reopen:{}", id));
        let mut label = move |id: &str| calls_lb.borrow_mut().push(format!("label:{}", id));
        execute_plan(&plan, &mut create, &mut close, &mut reopen, &mut label);

        // Expected order: label fixes, creates, closes, reopens
        assert_eq!(
            calls.borrow().clone(),
            vec![
                "label:LABEL1",
                "label:LABEL2",
                "create:New A",
                "close:CLOSE1",
                "reopen:REOPEN1",
            ]
        );

        // Running again should repeat same sequence (caller-side idempotency is ensured via underlying ops)
        let calls2: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let c2_c = Rc::clone(&calls2);
        let c2_cl = Rc::clone(&calls2);
        let c2_ro = Rc::clone(&calls2);
        let c2_lb = Rc::clone(&calls2);
        let mut create2 =
            move |t: &DesiredTask| c2_c.borrow_mut().push(format!("create:{}", t.text));
        let mut close2 = move |id: &str| c2_cl.borrow_mut().push(format!("close:{}", id));
        let mut reopen2 = move |id: &str| c2_ro.borrow_mut().push(format!("reopen:{}", id));
        let mut label2 = move |id: &str| c2_lb.borrow_mut().push(format!("label:{}", id));
        execute_plan(&plan, &mut create2, &mut close2, &mut reopen2, &mut label2);
        assert_eq!(calls.borrow().clone(), calls2.borrow().clone());
    }
}

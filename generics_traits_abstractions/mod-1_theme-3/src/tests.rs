use super::*;

#[test]
fn explicit() {
    let alice = dbg!(SimpleAccount::new("alice", 100));
    assert!(matches!(alice.get_balance(), Ok(100)));
    let bob = dbg!(SimpleAccount::new("bob", 200));
    assert!(matches!(bob.get_balance(), Ok(200)));
    let charlie = dbg!(SimpleAccount::new("charlie", 300));
    assert!(matches!(charlie.get_balance(), Ok(300)));

    assert!(matches!(
        SingleOperationTransaction::new(Deposit::new(alice.clone(), 50)).apply(),
        Ok(())
    ));
    assert!(
        matches!(alice.get_balance(), Ok(150)),
        "{:?}",
        alice.get_balance()
    );

    let res = SingleOperationTransaction::new(Withdrawal::new(bob.clone(), 250)).apply();
    assert!(
        matches!(
            &res,
            Err(TransactionError {
                #[allow(unused_variables)]
                account: bob,
                operation: TransactionOperation::Balance(BalanceOperation::Debit(250)),
                err: AccountError::InsufficientFunds,
            })
        ),
        "{:?}",
        res
    );
    assert!(
        matches!(bob.get_balance(), Ok(200)),
        "{:?}",
        bob.get_balance()
    );

    assert!(matches!(
        SingleOperationTransaction::new(Withdrawal::new(bob.clone(), 50)).apply(),
        Ok(())
    ));
    assert!(
        matches!(bob.get_balance(), Ok(150)),
        "{:?}",
        bob.get_balance()
    );

    assert!(matches!(
        SingleOperationTransaction::new(Transfer::new(charlie.clone(), alice.clone(), 50)).apply(),
        Ok(())
    ));
    assert!(
        matches!(charlie.get_balance(), Ok(250)),
        "{:?}",
        charlie.get_balance()
    );
    assert!(
        matches!(alice.get_balance(), Ok(200)),
        "{:?}",
        alice.get_balance()
    );

    assert!(matches!(
        PairedOperationTransaction::new(
            Transfer::new(alice.clone(), bob.clone(), 50),
            Transfer::new(bob.clone(), charlie.clone(), 50),
        )
        .apply(),
        Ok(())
    ));
    assert!(
        matches!(alice.get_balance(), Ok(150)),
        "{:?}",
        alice.get_balance()
    );
    assert!(
        matches!(bob.get_balance(), Ok(150)),
        "{:?}",
        alice.get_balance()
    );
    assert!(
        matches!(charlie.get_balance(), Ok(300)),
        "{:?}",
        charlie.get_balance()
    );

    assert!(matches!(
        MultipleOperationTransaction::new(vec![
            Box::new(Transfer::new(alice.clone(), bob.clone(), 50)),
            Box::new(Transfer::new(bob.clone(), charlie.clone(), 40)),
            Box::new(Transfer::new(charlie.clone(), alice.clone(), 30)),
        ])
        .apply(),
        Ok(())
    ));
    assert!(
        matches!(alice.get_balance(), Ok(130)),
        "{:?}",
        alice.get_balance()
    );
    assert!(
        matches!(bob.get_balance(), Ok(160)),
        "{:?}",
        bob.get_balance()
    );
    assert!(
        matches!(charlie.get_balance(), Ok(310)),
        "{:?}",
        charlie.get_balance()
    );
}

#[test]
fn implicit() {
    let alice = dbg!(SimpleAccount::new("alice", 100));
    assert!(matches!(alice.get_balance(), Ok(100)));
    let bob = dbg!(SimpleAccount::new("bob", 200));
    assert!(matches!(bob.get_balance(), Ok(200)));
    let charlie = dbg!(SimpleAccount::new("charlie", 300));
    assert!(matches!(charlie.get_balance(), Ok(300)));

    assert!(matches!((alice.clone() + 50.into()).apply(), Ok(())));
    assert!(
        matches!(alice.get_balance(), Ok(150)),
        "{:?}",
        alice.get_balance()
    );

    assert!(matches!((bob.clone() - 50.into()).apply(), Ok(())));
    assert!(
        matches!(bob.get_balance(), Ok(150)),
        "{:?}",
        bob.get_balance()
    );

    assert!(matches!(
        transaction!( charlie => 50 => alice ).apply(),
        Ok(())
    ));
    assert!(
        matches!(charlie.get_balance(), Ok(250)),
        "{:?}",
        charlie.get_balance()
    );
    assert!(
        matches!(alice.get_balance(), Ok(200)),
        "{:?}",
        alice.get_balance()
    );

    assert!(matches!(
        transaction![charlie.clone() + 50.into(), alice.clone() - 50.into()].apply(),
        Ok(())
    ));
    assert!(
        matches!(alice.get_balance(), Ok(150)),
        "{:?}",
        alice.get_balance()
    );
    assert!(
        matches!(charlie.get_balance(), Ok(300)),
        "{:?}",
        charlie.get_balance()
    );
}

use std::sync::Arc;

fn main() {
    println!("Hello, world!");
}

enum AccountError {
    AccountIsBlocked,
    InsufficientFunds,
}

// #[derive(Hash, PartialEq, Eq)]
// struct AccountId(u64);

trait Account<AccountId: std::hash::Hash + PartialEq + Eq> {
    fn id(&self) -> AccountId;
    //
    fn lock(&self) -> Result<(), AccountError>;
    fn unlock(&self);
    //
    fn debit(&self, amount: Amount) -> Result<(), AccountError>;
    fn credit(&self, amount: Amount);
}

trait Operation<AccountId: std::hash::Hash + PartialEq + Eq> {
    fn debit_account(&self) -> Option<(Arc<Box<dyn Account<AccountId>>>, Amount)> {
        None
    }
    fn credit_account(&self) -> Option<(Arc<Box<dyn Account<AccountId>>>, Amount)> {
        None
    }
}

#[derive(Clone, Copy)]
struct Amount(u64);

impl<AccountId: std::hash::Hash + PartialEq + Eq> Operation<AccountId> for Withdrawal<AccountId> {
    fn debit_account(&self) -> Option<(Arc<Box<dyn Account<AccountId>>>, Amount)> {
        Some((Arc::clone(&self.account), self.debit))
    }
}

impl<AccountId: std::hash::Hash + PartialEq + Eq> Operation<AccountId> for Deposit<AccountId> {
    fn credit_account(&self) -> Option<(Arc<Box<dyn Account<AccountId>>>, Amount)> {
        Some((Arc::clone(&self.account), self.credit))
    }
}

impl<AccountId: std::hash::Hash + PartialEq + Eq> Operation<AccountId> for Transfer<AccountId> {
    fn debit_account(&self) -> Option<(Arc<Box<dyn Account<AccountId>>>, Amount)> {
        Some((Arc::clone(&self.withdrawal.account), self.withdrawal.debit))
    }
    fn credit_account(&self) -> Option<(Arc<Box<dyn Account<AccountId>>>, Amount)> {
        Some((Arc::clone(&self.deposit.account), self.deposit.credit))
    }
}

struct TransactionError<AccountId: std::hash::Hash + PartialEq + Eq> {
    account: Arc<Box<dyn Account<AccountId>>>,
    err: AccountError,
}

trait Transaction<AccountId: std::hash::Hash + PartialEq + Eq> {
    fn apply(&self) -> Result<(), TransactionError<AccountId>> {
        let debit_accounts = self.debit_accounts();
        let credit_accounts = self.credit_accounts();
        let unique_accounts = {
            let mut count_of_succeffully_locked_accounts = 0;
            use itertools::Itertools;
            let unique_accounts = debit_accounts
                .iter()
                .map(|(account, _amount)| Arc::clone(&account))
                .chain(
                    credit_accounts
                        .iter()
                        .map(|(account, _amount)| Arc::clone(&account)),
                )
                .unique_by(|account| account.id())
                .collect::<Vec<_>>();
            for account in unique_accounts.iter() {
                if let Err(err) = account.lock() {
                    for account in unique_accounts
                        .iter()
                        .take(count_of_succeffully_locked_accounts)
                    {
                        account.unlock();
                    }
                    return Err(TransactionError {
                        account: Arc::clone(account),
                        err,
                    });
                } else {
                    count_of_succeffully_locked_accounts += 1;
                }
            }
            unique_accounts
        };
        {
            let mut count_of_successfully_debited_accounts = 0;
            for (account, amount) in debit_accounts.iter() {
                if let Err(err) = account.debit(*amount) {
                    for (account, amount) in debit_accounts
                        .iter()
                        .take(count_of_successfully_debited_accounts)
                    {
                        account.credit(*amount);
                    }
                    return Err(TransactionError {
                        account: Arc::clone(account),
                        err,
                    });
                } else {
                    count_of_successfully_debited_accounts += 1;
                }
            }
        }
        for (account, amount) in credit_accounts.iter() {
            account.credit(*amount);
        }
        for account in unique_accounts {
            account.unlock();
        }
        Ok(())
    }
    fn debit_accounts(&self) -> Vec<(Arc<Box<dyn Account<AccountId>>>, Amount)>;
    fn credit_accounts(&self) -> Vec<(Arc<Box<dyn Account<AccountId>>>, Amount)>;
}

// struct SingleOperationTransaction<AccountId: std::hash::Hash + PartialEq + Eq>
// // <T: Operation<AccountId>>
// // where
// //     AccountId: std::hash::Hash + PartialEq + Eq, // <
// //     // AccountId: Operation<AccountId>,
// //     AccountId: std::hash::Hash + PartialEq + Eq,
// //     // AccountId: std::hash::Hash + PartialEq + Eq,
// // >
// {
//     operation: Operation<AccountId>,
//     // operation: Box<dyn Operation<AccountId>>,
// }

// struct SingleOperationTransaction<T: Operation<AccountId: std::hash::Hash + PartialEq + Eq>> {
//     operation: T,
// }
//
struct SingleOperationTransaction<T, AccountId>
where
    T: Operation<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    operation: T,
    marker: std::marker::PhantomData<AccountId>,
}
//
struct PairedOperationTransaction<T1, T2, AccountId>
where
    T1: Operation<AccountId>,
    T2: Operation<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    first_operation: T1,
    second_operation: T2,
    marker: std::marker::PhantomData<AccountId>,
}

//

struct Withdrawal<AccountId: std::hash::Hash + PartialEq + Eq> {
    account: Arc<Box<dyn Account<AccountId>>>,
    debit: Amount,
}

struct Deposit<AccountId: std::hash::Hash + PartialEq + Eq> {
    account: Arc<Box<dyn Account<AccountId>>>,
    credit: Amount,
}

struct Transfer<AccountId: std::hash::Hash + PartialEq + Eq> {
    withdrawal: Withdrawal<AccountId>,
    deposit: Deposit<AccountId>,
}

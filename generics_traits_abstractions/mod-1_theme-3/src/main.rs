fn main() {
    println!("Hello, world!");
}

// ----------------------------
//
enum AccountError {
    AccountIsAlreadyLocked,
    InsufficientFunds,
}

trait Account<AccountId: std::hash::Hash + PartialEq + Eq>: Clone {
    fn id(&self) -> AccountId;
    //
    fn lock(&self) -> Result<(), AccountError>;
    fn unlock(&self);
    //
    fn debit(&self, amount: Amount) -> Result<(), AccountError>;
    fn credit(&self, amount: Amount);
}

#[derive(Clone, Copy)]
struct Amount(u64);

// ----------------------------

trait Operation<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    fn debit_account(&self) -> Option<(&A, Amount)> {
        None
    }
    fn credit_account(&self) -> Option<(&A, Amount)> {
        None
    }
}

// ----------------------------

struct Withdrawal<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    account: A,
    debit: Amount,
    marker: std::marker::PhantomData<AccountId>,
}

impl<A, AccountId> Operation<A, AccountId> for Withdrawal<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    fn debit_account(&self) -> Option<(&A, Amount)> {
        Some((&self.account, self.debit))
    }
}

struct Deposit<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    account: A,
    credit: Amount,
    marker: std::marker::PhantomData<AccountId>,
}

impl<A, AccountId> Operation<A, AccountId> for Deposit<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    fn credit_account(&self) -> Option<(&A, Amount)> {
        Some((&self.account, self.credit))
    }
}

struct Transfer<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    withdrawal: Withdrawal<A, AccountId>,
    deposit: Deposit<A, AccountId>,
}

impl<A, AccountId> Operation<A, AccountId> for Transfer<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    fn debit_account(&self) -> Option<(&A, Amount)> {
        self.withdrawal.debit_account()
    }
    fn credit_account(&self) -> Option<(&A, Amount)> {
        self.deposit.credit_account()
    }
}

// ----------------------------

struct TransactionError<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    account: A,
    err: AccountError,
    marker: std::marker::PhantomData<AccountId>,
}

trait Transaction<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    fn apply(&self) -> Result<(), TransactionError<A, AccountId>> {
        let debit_accounts = self.debit_accounts();
        let credit_accounts = self.credit_accounts();
        let unique_accounts = {
            use itertools::Itertools;
            debit_accounts
                .iter()
                .map(|(account, _amount)| account)
                .chain(credit_accounts.iter().map(|(account, _amount)| account))
                .unique_by(|account| account.id())
                .collect::<Vec<_>>()
        };
        {
            let mut count_of_succeffully_locked_accounts = 0;
            for account in unique_accounts.iter() {
                if let Err(err) = account.lock() {
                    for account in unique_accounts
                        .iter()
                        .take(count_of_succeffully_locked_accounts)
                    {
                        account.unlock();
                    }
                    return Err(TransactionError {
                        account: (**account).clone(),
                        err,
                        marker: std::marker::PhantomData,
                    });
                } else {
                    count_of_succeffully_locked_accounts += 1;
                }
            }
        }
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
                        account: (**account).clone(),
                        err,
                        marker: std::marker::PhantomData,
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
    fn debit_accounts(&self) -> Vec<(&A, Amount)>;
    fn credit_accounts(&self) -> Vec<(&A, Amount)>;
}

// ----------------------------

struct SingleOperationTransaction<T, A, AccountId>
where
    T: Operation<A, AccountId>,
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    operation: T,
    marker_a: std::marker::PhantomData<A>,
    marker_account_id: std::marker::PhantomData<AccountId>,
}

impl<T, A, AccountId> Transaction<A, AccountId> for SingleOperationTransaction<T, A, AccountId>
where
    T: Operation<A, AccountId>,
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    fn debit_accounts(&self) -> Vec<(&A, Amount)> {
        if let Some((account, amount)) = self.operation.debit_account() {
            vec![(account, amount)]
        } else {
            vec![]
        }
    }
    fn credit_accounts(&self) -> Vec<(&A, Amount)> {
        if let Some((account, amount)) = self.operation.credit_account() {
            vec![(account, amount)]
        } else {
            vec![]
        }
    }
}

// ----------------------------

struct PairedOperationTransaction<T1, T2, A, AccountId>
where
    T1: Operation<A, AccountId>,
    T1: Operation<A, AccountId>,
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    first_operation: T1,
    second_operation: T2,
    marker_a: std::marker::PhantomData<A>,
    marker_account_id: std::marker::PhantomData<AccountId>,
}

impl<T1, T2, A, AccountId> Transaction<A, AccountId>
    for PairedOperationTransaction<T1, T2, A, AccountId>
where
    T1: Operation<A, AccountId>,
    T2: Operation<A, AccountId>,
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    fn debit_accounts(&self) -> Vec<(&A, Amount)> {
        vec![
            self.first_operation.debit_account(),
            self.second_operation.debit_account(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
    }
    fn credit_accounts(&self) -> Vec<(&A, Amount)> {
        vec![
            self.first_operation.credit_account(),
            self.second_operation.credit_account(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
    }
}

// ----------------------------

struct MultipleOperationTransaction<A, AccountId>
where
    A: Account<AccountId>,
    AccountId: std::hash::Hash + PartialEq + Eq,
{
    operations: Vec<Box<dyn Operation<A, AccountId>>>,
    marker_account_id: std::marker::PhantomData<AccountId>,
}

// #[cfg(test)]
// mod tests {

// struct SimpleAccount {
//     id: u64,
//     is_locked: std::sync::atomic::AtomicBool,
//     balance: std::sync::atomic::AtomicU64,
// }
// impl Account<u64> for std::sync::Arc<SimpleAccount> {
//     fn id(&self) -> u64 {
//         self.id
//     }
//     fn lock(&self) -> Result<(), AccountError> {
//         if self
//             .is_locked
//             .swap(true, std::sync::atomic::Ordering::SeqCst)
//         {
//             Err(AccountError::AccountIsAlreadyLocked)
//         } else {
//             Ok(())
//         }
//     }
//     fn unlock(&self) {
//         self.is_locked
//             .store(false, std::sync::atomic::Ordering::SeqCst)
//     }
//     fn debit(&self, amount: Amount) -> Result<(), AccountError> {
//         if
//         self.balance.fetch_add(amount.0)
//     }
//     fn credit(&self, amount: Amount) {
//         self.balance.fetch_add(amount.0)
//     }
//     fn get_balance(&self) -> u64 {}
// }

// }

fn main() {
    println!("Hello, world!");
}

// ----------------------------

#[derive(Debug, Clone, Copy)]
struct Amount(u64);

// ----------------------------

#[derive(Debug)]
enum AccountError {
    AlreadyLocked,
    InsufficientFunds,
}

trait Account: Clone {
    type AccountId: std::hash::Hash + Eq;
    type LockedAccount: LockedAccount;

    fn id(&self) -> Self::AccountId;
    fn get_locked(&self) -> Result<std::sync::Arc<Self::LockedAccount>, AccountError>;
}

trait LockedAccount {
    fn debit(&self, amount: Amount) -> Result<(), AccountError>;
    fn credit(&self, amount: Amount);
}

// ----------------------------

trait Operation<T: Account> {
    fn debit_account(&self) -> Option<(&T, Amount)> {
        None
    }
    fn credit_account(&self) -> Option<(&T, Amount)> {
        None
    }
}

// ----------------------------

struct Withdrawal<T: Account> {
    account: T,
    debit: Amount,
}

impl<T: Account> Operation<T> for Withdrawal<T> {
    fn debit_account(&self) -> Option<(&T, Amount)> {
        Some((&self.account, self.debit))
    }
}

// ----------------------------

struct Deposit<T: Account> {
    account: T,
    credit: Amount,
}

impl<T: Account> Operation<T> for Deposit<T> {
    fn credit_account(&self) -> Option<(&T, Amount)> {
        Some((&self.account, self.credit))
    }
}

struct Transfer<T: Account> {
    withdrawal: Withdrawal<T>,
    deposit: Deposit<T>,
}

impl<T: Account> Operation<T> for Transfer<T> {
    fn debit_account(&self) -> Option<(&T, Amount)> {
        self.withdrawal.debit_account()
    }
    fn credit_account(&self) -> Option<(&T, Amount)> {
        self.deposit.credit_account()
    }
}

// ----------------------------

struct TransactionError<T: Account> {
    account: T,
    err: AccountError,
}

trait Transaction {
    type Account: Account;

    fn debit_accounts(&self) -> Vec<(&Self::Account, Amount)>;
    fn credit_accounts(&self) -> Vec<(&Self::Account, Amount)>;

    fn apply(&self) -> Result<(), TransactionError<Self::Account>> {
        let mut locked_accounts = std::collections::HashMap::new();

        let locked_debit_accounts = {
            let mut locked_debit_accounts = vec![];
            for (account, amount) in self.debit_accounts() {
                use std::collections::hash_map::Entry;
                match locked_accounts.entry(account.id()) {
                    Entry::Occupied(e) => {
                        locked_debit_accounts.push((
                            account,
                            std::sync::Arc::clone(e.get()),
                            amount,
                        ));
                    }
                    Entry::Vacant(e) => match account.get_locked() {
                        Ok(locked_account) => {
                            e.insert(std::sync::Arc::clone(&locked_account));
                            locked_debit_accounts.push((account, locked_account, amount));
                        }
                        Err(err) => {
                            return Err(TransactionError {
                                account: account.clone(),
                                err,
                            });
                        }
                    },
                }
            }
            locked_debit_accounts
        };

        let locked_credit_accounts = {
            let mut locked_credit_accounts = vec![];
            for (account, amount) in self.credit_accounts() {
                use std::collections::hash_map::Entry;
                match locked_accounts.entry(account.id()) {
                    Entry::Occupied(e) => {
                        locked_credit_accounts.push((std::sync::Arc::clone(e.get()), amount));
                    }
                    Entry::Vacant(e) => match account.get_locked() {
                        Ok(locked_account) => {
                            e.insert(std::sync::Arc::clone(&locked_account));
                            locked_credit_accounts.push((locked_account, amount));
                        }
                        Err(err) => {
                            return Err(TransactionError {
                                account: account.clone(),
                                err,
                            });
                        }
                    },
                }
            }
            locked_credit_accounts
        };

        {
            let mut count_of_successfully_debited_accounts = 0;
            for (account, locked_account, amount) in locked_debit_accounts.iter() {
                if let Err(err) = locked_account.debit(*amount) {
                    for (account, locked_account, amount) in locked_debit_accounts
                        .iter()
                        .take(count_of_successfully_debited_accounts)
                    {
                        locked_account.credit(*amount);
                    }
                    return Err(TransactionError {
                        account: (**account).clone(),
                        err,
                    });
                } else {
                    count_of_successfully_debited_accounts += 1;
                }
            }
        }

        for (locked_account, amount) in locked_credit_accounts.iter() {
            locked_account.credit(*amount);
        }

        Ok(())
    }
}

// ----------------------------

struct SingleOperationTransaction<T, A>
where
    T: Operation<A>,
    A: Account,
{
    operation: T,
    marker_a: std::marker::PhantomData<A>,
}

impl<T, A> Transaction for SingleOperationTransaction<T, A>
where
    T: Operation<A>,
    A: Account,
{
    type Account = A;

    fn debit_accounts(&self) -> Vec<(&Self::Account, Amount)> {
        if let Some((account, amount)) = self.operation.debit_account() {
            vec![(account, amount)]
        } else {
            vec![]
        }
    }
    fn credit_accounts(&self) -> Vec<(&Self::Account, Amount)> {
        if let Some((account, amount)) = self.operation.credit_account() {
            vec![(account, amount)]
        } else {
            vec![]
        }
    }
}

// ----------------------------

struct PairedOperationTransaction<T1, T2, A>
where
    T1: Operation<A>,
    T1: Operation<A>,
    A: Account,
{
    first_operation: T1,
    second_operation: T2,
    marker_a: std::marker::PhantomData<A>,
}

impl<T1, T2, A> Transaction for PairedOperationTransaction<T1, T2, A>
where
    T1: Operation<A>,
    T2: Operation<A>,
    A: Account,
{
    type Account = A;

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

struct MultipleOperationTransaction<A>
where
    A: Account,
{
    operations: Vec<Box<dyn Operation<A>>>,
}

impl<A> Transaction for MultipleOperationTransaction<A>
where
    A: Account,
{
    type Account = A;

    fn debit_accounts(&self) -> Vec<(&A, Amount)> {
        self.operations
            .iter()
            .filter_map(|i| i.debit_account())
            .collect::<Vec<_>>()
    }
    fn credit_accounts(&self) -> Vec<(&A, Amount)> {
        self.operations
            .iter()
            .filter_map(|i| i.credit_account())
            .collect::<Vec<_>>()
    }
}

// ----------------------------

// #[cfg(test)]
// mod tests {

struct LockedSimpleAccount {
    parent: std::sync::Arc<SimpleAccount>,
}

impl LockedAccount for LockedSimpleAccount {
    fn debit(&self, amount: Amount) -> Result<(), AccountError> {
        todo!();
    }
    fn credit(&self, amount: Amount) {
        todo!();
    }
}

impl Drop for LockedSimpleAccount {
    fn drop(&mut self) {
        *self.parent.locked.write().unwrap() = None;
    }
}

struct SimpleAccount {
    id: u64,
    locked: std::sync::RwLock<Option<std::sync::Weak<LockedSimpleAccount>>>,
    balance: std::sync::atomic::AtomicU64,
}

impl LockedAccount for std::sync::Arc<SimpleAccount> {
    fn debit(&self, amount: Amount) -> Result<(), AccountError> {
        // if self.parent
        // if let Some(account) =
        todo!();
    }
    fn credit(&self, amount: Amount) {
        todo!();
    }
}

impl Account for std::sync::Arc<SimpleAccount> {
    type AccountId = u64;
    type LockedAccount = LockedSimpleAccount;
    fn id(&self) -> u64 {
        self.id
    }
    fn get_locked(&self) -> Result<std::sync::Arc<Self::LockedAccount>, AccountError> {
        if let Some(locked) = (*self.locked.read().unwrap())
            .as_ref()
            .and_then(|weak| weak.upgrade())
        {
            Err(AccountError::AlreadyLocked)
        } else {
            let locked = std::sync::Arc::new(LockedSimpleAccount {
                parent: std::sync::Arc::clone(&self),
            });
            *self.locked.write().unwrap() = Some(std::sync::Arc::downgrade(&locked));
            Ok(locked)
        }
    }
    // fn debit(&self, amount: Amount) -> Result<(), AccountError> {
    //     // if
    //     self.balance.fetch_add(amount.0)
    // }
    // fn credit(&self, amount: Amount) {
    //     self.balance.fetch_add(amount.0)
    // }
    // fn get_balance(&self) -> u64 {}
}

// }

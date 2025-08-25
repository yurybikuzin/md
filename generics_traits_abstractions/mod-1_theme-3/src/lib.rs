#[derive(Debug)]
pub enum AccountError {
    Locked,
    InsufficientFunds,
}

pub trait Account: Clone {
    type AccountId: std::hash::Hash + Eq;
    // type LockedAccount: LockedAccount<Self>;
    type LockedAccount: LockedAccount<Self::Amount>;
    type Amount: Clone + PartialOrd + std::ops::SubAssign + std::ops::AddAssign;

    fn id(&self) -> Self::AccountId;
    fn get_balance(&self) -> Result<Self::Amount, AccountError>;
    fn get_locked(&self) -> Result<Self::LockedAccount, AccountError>;
    // fn commit_locked(&self, locked: Self::LockedAccount);
}

pub trait LockedAccount<Amount> {
    fn debit(&mut self, amount: Amount) -> Result<(), AccountError>;
    fn credit(&mut self, amount: Amount) -> Result<(), AccountError>;
}

// ----------------------------

pub trait Operation<T: Account> {
    fn debit_account(&self) -> Option<(&T, T::Amount)> {
        None
    }
    fn credit_account(&self) -> Option<(&T, T::Amount)> {
        None
    }
}

// ----------------------------

pub struct Withdrawal<T: Account> {
    account: T,
    debit: T::Amount,
}

impl<T: Account> Operation<T> for Withdrawal<T> {
    fn debit_account(&self) -> Option<(&T, T::Amount)> {
        Some((&self.account, self.debit.clone()))
    }
}

// ----------------------------

pub struct Deposit<T: Account> {
    account: T,
    credit: T::Amount,
}

impl<T: Account> Operation<T> for Deposit<T> {
    fn credit_account(&self) -> Option<(&T, T::Amount)> {
        Some((&self.account, self.credit.clone()))
    }
}

pub struct Transfer<T: Account> {
    withdrawal: Withdrawal<T>,
    deposit: Deposit<T>,
}

impl<T: Account> Operation<T> for Transfer<T> {
    fn debit_account(&self) -> Option<(&T, T::Amount)> {
        self.withdrawal.debit_account()
    }
    fn credit_account(&self) -> Option<(&T, T::Amount)> {
        self.deposit.credit_account()
    }
}

// ----------------------------

#[derive(Debug)]
pub struct TransactionError<T: Account> {
    account: T,
    err: AccountError,
}

pub trait Transaction {
    type TransactionAccount: Account;

    fn debit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )>;
    fn credit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )>;

    fn apply(&self) -> Result<(), TransactionError<Self::TransactionAccount>> {
        use std::collections::HashMap;

        enum BalanceOperation<Amount> {
            Credit(Amount),
            Debit(Amount),
        }

        struct Value<T: Account> {
            account: T,
            locked: T::LockedAccount,
            balance_operations: Vec<BalanceOperation<T::Amount>>,
        }
        type LockedAccounts<AccountId, Account> = HashMap<AccountId, Value<Account>>;
        let locked_accounts = {
            let mut locked_accounts = LockedAccounts::<
                <Self::TransactionAccount as Account>::AccountId,
                Self::TransactionAccount,
            >::new();

            for (account, amount) in self.debit_accounts() {
                let balance_operation = BalanceOperation::Debit(amount);
                use std::collections::hash_map::Entry;
                match locked_accounts.entry(account.id()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().balance_operations.push(balance_operation);
                    }
                    Entry::Vacant(e) => {
                        let account = account.clone();
                        match account.get_locked() {
                            Ok(locked) => {
                                e.insert(Value {
                                    balance_operations: vec![balance_operation],
                                    locked,
                                    account,
                                });
                            }
                            Err(err) => {
                                return Err(TransactionError { account, err });
                            }
                        }
                    }
                }
            }

            for (account, amount) in self.credit_accounts() {
                let balance_operation = BalanceOperation::Credit(amount);
                use std::collections::hash_map::Entry;
                match locked_accounts.entry(account.id()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().balance_operations.push(balance_operation);
                    }
                    Entry::Vacant(e) => {
                        let account = account.clone();
                        match account.get_locked() {
                            Ok(locked) => {
                                e.insert(Value {
                                    balance_operations: vec![balance_operation],
                                    locked,
                                    account,
                                });
                            }
                            Err(err) => {
                                return Err(TransactionError { account, err });
                            }
                        }
                    }
                }
            }

            locked_accounts
        };

        let to_be_commited = {
            let mut to_be_commited = vec![];
            for (
                _,
                Value {
                    account,
                    mut locked,
                    balance_operations,
                },
            ) in locked_accounts
            {
                for i in balance_operations {
                    let res = match i {
                        BalanceOperation::Credit(amount) => locked.credit(amount),
                        BalanceOperation::Debit(amount) => locked.debit(amount),
                    };
                    if let Err(err) = res {
                        return Err(TransactionError { err, account });
                    }
                }
                to_be_commited.push((account, locked));
            }

            to_be_commited
        };

        todo!();
        // for (account, locked) in to_be_commited {
        //     account.commit_locked(locked);
        // }

        Ok(())
    }
}

// ----------------------------

pub struct SingleOperationTransaction<T, A>
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
    type TransactionAccount = A;

    fn debit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )> {
        if let Some((account, amount)) = self.operation.debit_account() {
            vec![(account, amount)]
        } else {
            vec![]
        }
    }

    fn credit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )> {
        if let Some((account, amount)) = self.operation.credit_account() {
            vec![(account, amount)]
        } else {
            vec![]
        }
    }
}

// ----------------------------

pub struct PairedOperationTransaction<T1, T2, A>
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
    type TransactionAccount = A;

    fn debit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )> {
        vec![
            self.first_operation.debit_account(),
            self.second_operation.debit_account(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
    }

    fn credit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )> {
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

pub struct MultipleOperationTransaction<A>
where
    A: Account,
{
    operations: Vec<Box<dyn Operation<A>>>,
}

impl<A> Transaction for MultipleOperationTransaction<A>
where
    A: Account,
{
    type TransactionAccount = A;

    fn debit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )> {
        self.operations
            .iter()
            .filter_map(|i| i.debit_account())
            .collect::<Vec<_>>()
    }
    fn credit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )> {
        self.operations
            .iter()
            .filter_map(|i| i.credit_account())
            .collect::<Vec<_>>()
    }
}

// ----------------------------

// #[cfg(test)]
// mod tests {

// pub struct LockedSimpleAccount<'a, SimpleAccount: Account> {
//     non_commited_balance: SimpleAccount::Amount,
//     guard: std::sync::RwLockWriteGuard<'a, SimpleAccount::Amount>,
//     // balance: SimpleAccount::Amount,
//     // parent: SimpleAccount,
// }

// impl<'a, T: Account<'a>> LockedAccount<T> for LockedSimpleAccount<'a, T> {
//     fn debit(&mut self, amount: T::Amount) -> Result<(), AccountError> {
//         if amount > self.non_commited_balance {
//             Err(AccountError::InsufficientFunds)
//         } else {
//             self.non_commited_balance -= amount;
//             Ok(())
//         }
//     }
//     fn credit(&mut self, amount: T::Amount) -> Result<(), AccountError> {
//         self.non_commited_balance += amount;
//         Ok(())
//     }
// }

pub struct LockedSimpleAccount<'a, Amount> {
    non_commited_balance: Amount,
    guard: std::sync::RwLockWriteGuard<'a, Amount>,
}

impl<'a, Amount: Clone + PartialOrd + std::ops::SubAssign + std::ops::AddAssign>
    LockedAccount<Amount> for LockedSimpleAccount<'a, Amount>
{
    fn debit(&mut self, amount: Amount) -> Result<(), AccountError> {
        if amount > self.non_commited_balance {
            Err(AccountError::InsufficientFunds)
        } else {
            self.non_commited_balance -= amount;
            Ok(())
        }
    }
    fn credit(&mut self, amount: Amount) -> Result<(), AccountError> {
        self.non_commited_balance += amount;
        Ok(())
    }
}

pub struct SimpleAccount<
    AccountId: Clone + std::hash::Hash + PartialEq + Eq,
    Amount: Clone + PartialOrd + std::ops::SubAssign + std::ops::AddAssign,
> {
    id: AccountId,
    balance: std::sync::RwLock<Amount>,
    // internal: std::sync::RwLock<SimpleAccountInternal<std::sync::Arc<Self>>>,
    marker: std::marker::PhantomData<Amount>,
}

// pub struct SimpleAccount<
//     AccountId: Clone + std::hash::Hash + PartialEq + Eq,
//     Amount: Clone + PartialOrd + std::ops::SubAssign + std::ops::AddAssign,
// > {
//     id: AccountId,
//     internal: std::sync::RwLock<SimpleAccountInternal<std::sync::Arc<Self>>>,
//     marker: std::marker::PhantomData<Amount>,
// }

impl<
        AccountId: Clone + std::hash::Hash + PartialEq + Eq,
        Amount: Clone + PartialOrd + std::ops::SubAssign + std::ops::AddAssign,
    > Account for std::sync::Arc<SimpleAccount<AccountId, Amount>>
{
    type AccountId = AccountId;
    type LockedAccount = LockedSimpleAccount<Self::Amount>;
    type Amount = Amount;
    fn id(&self) -> AccountId {
        self.id.clone()
    }
    fn get_locked(&self) -> Result<Self::LockedAccount, AccountError> {
        if let Ok(mut guard) = self.balance.write() {
            Ok(LockedSimpleAccount {
                non_commited_balance: guard.clone(),
                guard,
            })
        } else {
            Err(AccountError::Locked)
        }
        // if let Ok(mut internal) = self.internal.write() {
        //     let internal = &mut *internal;
        //     let mut value = SimpleAccountInternal::Locked;
        //     std::mem::swap(&mut value, internal);
        //     if let SimpleAccountInternal::Balance(balance) = value {
        //         Ok(LockedSimpleAccount {
        //             balance,
        //             // parent: std::sync::Arc::clone(self),
        //         })
        //     } else {
        //         Err(AccountError::Locked)
        //     }
        // } else {
        //     Err(AccountError::Locked)
        // }
    }
    // fn commit_locked(&self, locked: Self::LockedAccount) {
    //     *self.internal.write().unwrap() = SimpleAccountInternal::Balance(locked.balance);
    // }
    fn get_balance(&self) -> Result<Self::Amount, AccountError> {
        self.balance
            .read()
            .map(|guard| guard.clone())
            .map_err(|_| AccountError::Locked)
    }
}

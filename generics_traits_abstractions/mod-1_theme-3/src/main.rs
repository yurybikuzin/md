fn main() {
    println!("Hello, world!");
}

// ----------------------------

#[derive(Debug)]
enum AccountError {
    AlreadyLocked,
    InsufficientFunds,
}

trait Account: Clone {
    type AccountId: std::hash::Hash + Eq;
    type LockedAccount: LockedAccount<Self>;
    type Amount: Clone;

    fn id(&self) -> Self::AccountId;
    fn get_locked(&self) -> Result<Self::LockedAccount, AccountError>;
}

trait LockedAccount<T: Account> {
    fn debit(&mut self, amount: T::Amount) -> Result<(), AccountError>;
    fn credit(&mut self, amount: T::Amount) -> Result<(), AccountError>;
    fn commit(&self);
}

// ----------------------------

trait Operation<T: Account> {
    fn debit_account(&self) -> Option<(&T, T::Amount)> {
        None
    }
    fn credit_account(&self) -> Option<(&T, T::Amount)> {
        None
    }
}

// ----------------------------

struct Withdrawal<T: Account> {
    account: T,
    debit: T::Amount,
}

impl<T: Account> Operation<T> for Withdrawal<T> {
    fn debit_account(&self) -> Option<(&T, T::Amount)> {
        Some((&self.account, self.debit.clone()))
    }
}

// ----------------------------

struct Deposit<T: Account> {
    account: T,
    credit: T::Amount,
}

impl<T: Account> Operation<T> for Deposit<T> {
    fn credit_account(&self) -> Option<(&T, T::Amount)> {
        Some((&self.account, self.credit.clone()))
    }
}

struct Transfer<T: Account> {
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

struct TransactionError<T: Account> {
    account: T,
    err: AccountError,
}

trait Transaction {
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

        let mut to_be_commited = {
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
                to_be_commited.push(locked);
            }

            to_be_commited
        };

        for i in to_be_commited {
            i.commit();
        }

        Ok(())
    }
}
//
// // ----------------------------
//
// struct SingleOperationTransaction<T, A>
// where
//     T: Operation<A>,
//     A: Account,
// {
//     operation: T,
//     marker_a: std::marker::PhantomData<A>,
// }
//
// impl<T, A> Transaction for SingleOperationTransaction<T, A>
// where
//     T: Operation<A>,
//     A: Account,
// {
//     type Account = A;
//
//     fn debit_accounts(&self) -> Vec<(&Self::Account, Amount)> {
//         if let Some((account, amount)) = self.operation.debit_account() {
//             vec![(account, amount)]
//         } else {
//             vec![]
//         }
//     }
//     fn credit_accounts(&self) -> Vec<(&Self::Account, Amount)> {
//         if let Some((account, amount)) = self.operation.credit_account() {
//             vec![(account, amount)]
//         } else {
//             vec![]
//         }
//     }
// }
//
// // ----------------------------
//
// struct PairedOperationTransaction<T1, T2, A>
// where
//     T1: Operation<A>,
//     T1: Operation<A>,
//     A: Account,
// {
//     first_operation: T1,
//     second_operation: T2,
//     marker_a: std::marker::PhantomData<A>,
// }
//
// impl<T1, T2, A> Transaction for PairedOperationTransaction<T1, T2, A>
// where
//     T1: Operation<A>,
//     T2: Operation<A>,
//     A: Account,
// {
//     type Account = A;
//
//     fn debit_accounts(&self) -> Vec<(&A, Amount)> {
//         vec![
//             self.first_operation.debit_account(),
//             self.second_operation.debit_account(),
//         ]
//         .into_iter()
//         .flatten()
//         .collect::<Vec<_>>()
//     }
//     fn credit_accounts(&self) -> Vec<(&A, Amount)> {
//         vec![
//             self.first_operation.credit_account(),
//             self.second_operation.credit_account(),
//         ]
//         .into_iter()
//         .flatten()
//         .collect::<Vec<_>>()
//     }
// }
//
// // ----------------------------
//
// struct MultipleOperationTransaction<A>
// where
//     A: Account,
// {
//     operations: Vec<Box<dyn Operation<A>>>,
// }
//
// impl<A> Transaction for MultipleOperationTransaction<A>
// where
//     A: Account,
// {
//     type Account = A;
//
//     fn debit_accounts(&self) -> Vec<(&A, Amount)> {
//         self.operations
//             .iter()
//             .filter_map(|i| i.debit_account())
//             .collect::<Vec<_>>()
//     }
//     fn credit_accounts(&self) -> Vec<(&A, Amount)> {
//         self.operations
//             .iter()
//             .filter_map(|i| i.credit_account())
//             .collect::<Vec<_>>()
//     }
// }
//
// // ----------------------------
//
// // #[cfg(test)]
// // mod tests {
//
// struct LockedSimpleAccount {
//     parent: std::sync::Arc<SimpleAccount>,
// }
//
// impl LockedAccount for LockedSimpleAccount {
//     fn debit(&self, amount: Amount) -> Result<(), AccountError> {
//         todo!();
//     }
//     fn credit(&self, amount: Amount) {
//         todo!();
//     }
// }
//
// impl Drop for LockedSimpleAccount {
//     fn drop(&mut self) {
//         *self.parent.locked.write().unwrap() = None;
//     }
// }
//
// struct SimpleAccount {
//     id: u64,
//     locked: std::sync::RwLock<Option<std::sync::Weak<LockedSimpleAccount>>>,
//     balance: std::sync::atomic::AtomicU64,
// }
//
// impl LockedAccount for std::sync::Arc<SimpleAccount> {
//     fn debit(&self, amount: Amount) -> Result<(), AccountError> {
//         // if self.parent
//         // if let Some(account) =
//         todo!();
//     }
//     fn credit(&self, amount: Amount) {
//         todo!();
//     }
// }
//
// impl Account for std::sync::Arc<SimpleAccount> {
//     type AccountId = u64;
//     type LockedAccount = LockedSimpleAccount;
//     fn id(&self) -> u64 {
//         self.id
//     }
//     fn get_locked(&self) -> Result<std::sync::Arc<Self::LockedAccount>, AccountError> {
//         if let Some(locked) = (*self.locked.read().unwrap())
//             .as_ref()
//             .and_then(|weak| weak.upgrade())
//         {
//             Err(AccountError::AlreadyLocked)
//         } else {
//             let locked = std::sync::Arc::new(LockedSimpleAccount {
//                 parent: std::sync::Arc::clone(&self),
//             });
//             *self.locked.write().unwrap() = Some(std::sync::Arc::downgrade(&locked));
//             Ok(locked)
//         }
//     }
//     // fn debit(&self, amount: Amount) -> Result<(), AccountError> {
//     //     // if
//     //     self.balance.fetch_add(amount.0)
//     // }
//     // fn credit(&self, amount: Amount) {
//     //     self.balance.fetch_add(amount.0)
//     // }
//     // fn get_balance(&self) -> u64 {}
// }
//
// // }

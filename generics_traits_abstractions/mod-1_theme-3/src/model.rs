use super::*;

#[derive(Debug)]
pub enum AccountError {
    Locked,
    InsufficientFunds,
}

pub trait Account: Debug + Clone {
    type AccountId: Debug + Hash + Eq;
    type LockedAccount<'a>: LockedAccount<'a, Self>
    where
        Self: 'a;
    type Amount: Debug + Clone + PartialOrd + std::ops::SubAssign + std::ops::AddAssign;

    fn id(&self) -> Self::AccountId;
    fn get_balance(&self) -> Result<Self::Amount, AccountError>;
    fn get_locked<'a>(&'a self) -> Result<Self::LockedAccount<'a>, AccountError>;
}

pub trait LockedAccount<'a, T: Account>: Debug {
    fn debit(&mut self, amount: T::Amount) -> Result<(), AccountError>;
    fn credit(&mut self, amount: T::Amount) -> Result<(), AccountError>;
    fn commit(self);
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct TransactionError<T: Account> {
    pub account: T,
    pub operation: TransactionOperation<T::Amount>,
    pub err: AccountError,
}

#[derive(Debug)]
pub enum TransactionOperation<Amount> {
    Lock,
    Balance(BalanceOperation<Amount>),
}

#[derive(Debug)]
pub enum BalanceOperation<Amount> {
    Credit(Amount),
    Debit(Amount),
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

        #[derive(Debug)]
        struct Value<'a, T: Account + 'a> {
            account: T,
            locked: T::LockedAccount<'a>,
            balance_operations: Vec<BalanceOperation<T::Amount>>,
        }
        type LockedAccounts<'a, AccountId, Account> = HashMap<AccountId, Value<'a, Account>>;

        let locked_accounts = {
            let mut locked_accounts = LockedAccounts::<
                <Self::TransactionAccount as Account>::AccountId,
                Self::TransactionAccount,
            >::new();

            for (account, amount) in self.debit_accounts() {
                let balance_operation = BalanceOperation::Debit(amount.clone());
                use std::collections::hash_map::Entry;
                match locked_accounts.entry(account.id()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().balance_operations.push(balance_operation);
                    }
                    Entry::Vacant(e) => match account.get_locked() {
                        Ok(locked) => {
                            e.insert(Value {
                                balance_operations: vec![balance_operation],
                                locked,
                                account: (*account).clone(),
                            });
                        }
                        Err(err) => {
                            return Err(TransactionError {
                                account: (*account).clone(),
                                operation: TransactionOperation::Lock,
                                err,
                            });
                        }
                    },
                }
            }

            for (account, amount) in self.credit_accounts() {
                let balance_operation = BalanceOperation::Credit(amount);
                use std::collections::hash_map::Entry;
                match locked_accounts.entry(account.id()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().balance_operations.push(balance_operation);
                    }
                    Entry::Vacant(e) => match account.get_locked() {
                        Ok(locked) => {
                            e.insert(Value {
                                balance_operations: vec![balance_operation],
                                locked,
                                account: (*account).clone(),
                            });
                        }
                        Err(err) => {
                            return Err(TransactionError {
                                account: (*account).clone(),
                                operation: TransactionOperation::Balance(balance_operation),
                                err,
                            });
                        }
                    },
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
                for balance_operation in balance_operations {
                    let res = match &balance_operation {
                        BalanceOperation::Credit(amount) => locked.credit(amount.clone()),
                        BalanceOperation::Debit(amount) => locked.debit(amount.clone()),
                    };
                    if let Err(err) = res {
                        return Err(TransactionError {
                            err,
                            operation: TransactionOperation::Balance(balance_operation),
                            account,
                        });
                    }
                }
                to_be_commited.push(locked);
            }

            to_be_commited
        };

        for locked in to_be_commited {
            locked.commit();
        }

        Ok(())
    }
}

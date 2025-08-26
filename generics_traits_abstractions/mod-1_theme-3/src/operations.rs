use super::*;

// ----------------------------

pub struct Withdrawal<A: Account> {
    account: A,
    debit: A::Amount,
}

impl<A: Account> Withdrawal<A> {
    pub fn new(account: A, debit: A::Amount) -> Self {
        Self { account, debit }
    }
}

impl<A: Account> AccountBalanceOperation<A> for Withdrawal<A> {
    fn balance_operation(&self) -> (&A, BalanceOperation<A::Amount>) {
        (&self.account, BalanceOperation::Debit(self.debit.clone()))
    }
}

impl<A: Account> Transaction for Withdrawal<A> {
    type TransactionAccount = A;

    fn account_balance_operations(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        BalanceOperation<<Self::TransactionAccount as Account>::Amount>,
    )> {
        vec![self.balance_operation()]
    }
}

// ----------------------------

pub struct Deposit<A: Account> {
    account: A,
    credit: A::Amount,
}

impl<A: Account> Deposit<A> {
    pub fn new(account: A, credit: A::Amount) -> Self {
        Self { account, credit }
    }
}

impl<A: Account> AccountBalanceOperation<A> for Deposit<A> {
    fn balance_operation(&self) -> (&A, BalanceOperation<A::Amount>) {
        (&self.account, BalanceOperation::Credit(self.credit.clone()))
    }
}

impl<A> Transaction for Deposit<A>
where
    A: Account,
{
    type TransactionAccount = A;

    fn account_balance_operations(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        BalanceOperation<<Self::TransactionAccount as Account>::Amount>,
    )> {
        vec![self.balance_operation()]
    }
}

// ----------------------------

// #[derive(Transaction)]
pub struct Transfer<A: Account> {
    withdrawal: Withdrawal<A>,
    deposit: Deposit<A>,
}

impl<A: Account> Transfer<A> {
    pub fn new(from_account: A, to_account: A, amount: A::Amount) -> Self {
        Self {
            withdrawal: Withdrawal::new(from_account, amount.clone()),
            deposit: Deposit::new(to_account, amount),
        }
    }
}

// TODO: implement by #[derive(Transaction)]
impl<A> Transaction for Transfer<A>
where
    A: Account,
{
    type TransactionAccount = A;

    fn account_balance_operations(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        BalanceOperation<<Self::TransactionAccount as Account>::Amount>,
    )> {
        let mut ret = vec![];
        ret.extend(self.withdrawal.account_balance_operations());
        ret.extend(self.deposit.account_balance_operations());
        ret
    }
}

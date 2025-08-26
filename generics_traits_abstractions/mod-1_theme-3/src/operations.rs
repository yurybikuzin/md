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

impl<A: Account> Operation<A> for Withdrawal<A> {
    fn balance_operation(&self) -> (&A, BalanceOperation<A::Amount>) {
        (&self.account, BalanceOperation::Debit(self.debit.clone()))
    }
}

impl<A> Transaction for Withdrawal<A>
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

pub struct Deposit<A: Account> {
    account: A,
    credit: A::Amount,
}

impl<A: Account> Deposit<A> {
    pub fn new(account: A, credit: A::Amount) -> Self {
        Self { account, credit }
    }
}

impl<A: Account> Operation<A> for Deposit<A> {
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

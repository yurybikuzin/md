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
    fn debit_account(&self) -> Option<(&A, A::Amount)> {
        Some((&self.account, self.debit.clone()))
    }
}

impl<A> Transaction for Withdrawal<A>
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
        vec![(&self.account, self.debit.clone())]
    }

    fn credit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )> {
        vec![]
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
    fn credit_account(&self) -> Option<(&A, A::Amount)> {
        Some((&self.account, self.credit.clone()))
    }
}

impl<A> Transaction for Deposit<A>
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
        vec![]
    }

    fn credit_accounts(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        <Self::TransactionAccount as Account>::Amount,
    )> {
        vec![(&self.account, self.credit.clone())]
    }
}

// ----------------------------

pub struct Transfer<A: Account> {
    from_account: A,
    to_account: A,
    amount: A::Amount,
}

impl<A: Account> Transfer<A> {
    pub fn new(from_account: A, to_account: A, amount: A::Amount) -> Self {
        Self {
            from_account,
            to_account,
            amount,
        }
    }
}

impl<A: Account> Operation<A> for Transfer<A> {
    fn debit_account(&self) -> Option<(&A, A::Amount)> {
        Some((&self.from_account, self.amount.clone()))
    }
    fn credit_account(&self) -> Option<(&A, A::Amount)> {
        Some((&self.to_account, self.amount.clone()))
    }
}

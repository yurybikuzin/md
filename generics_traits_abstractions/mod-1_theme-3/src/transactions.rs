use super::*;

pub struct SingleOperationTransaction<T, A>
where
    T: Operation<A>,
    A: Account,
{
    operation: T,
    marker: PhantomData<A>,
}

impl<T: Operation<A>, A: Account> SingleOperationTransaction<T, A> {
    pub fn new(operation: T) -> Self {
        Self {
            operation,
            marker: PhantomData,
        }
    }
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
    marker: std::marker::PhantomData<A>,
}

impl<T1: Operation<A>, T2: Operation<A>, A: Account> PairedOperationTransaction<T1, T2, A> {
    pub fn new(first_operation: T1, second_operation: T2) -> Self {
        Self {
            first_operation,
            second_operation,
            marker: PhantomData,
        }
    }
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

pub struct MultipleOperationTransaction<T: Account> {
    operations: Vec<Box<dyn Operation<T>>>,
}

impl<T: Account> MultipleOperationTransaction<T> {
    pub fn new(operations: Vec<Box<dyn Operation<T>>>) -> Self {
        Self { operations }
    }
}

impl<T> Transaction for MultipleOperationTransaction<T>
where
    T: Account,
{
    type TransactionAccount = T;

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

#[macro_export]
macro_rules! transaction {
    ($from:expr => $amount:expr => $to:expr) => {
        SingleOperationTransaction::new(Transfer::new($from.clone(), $to.clone(), $amount))
    }; // (transfer $amount:expr from $from:expr to $to:expr) => {
    //     SingleOperationTransaction::new(Transfer::new($from.clone(), $to.clone(), $amount))
    // };
    // https://lukaswirth.dev/tlborm/decl-macros/patterns/push-down-acc.html
    // https://lukaswirth.dev/tlborm/decl-macros/patterns/tt-muncher.html
    ($operation:expr) => {
        ($operation)
    };
}
// pub use transaction;
// https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files

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

    fn account_balance_operations(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        BalanceOperation<<Self::TransactionAccount as Account>::Amount>,
    )> {
        vec![self.operation.balance_operation()]
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

    fn account_balance_operations(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        BalanceOperation<<Self::TransactionAccount as Account>::Amount>,
    )> {
        vec![
            self.first_operation.balance_operation(),
            self.second_operation.balance_operation(),
        ]
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

    fn account_balance_operations(
        &self,
    ) -> Vec<(
        &Self::TransactionAccount,
        BalanceOperation<<Self::TransactionAccount as Account>::Amount>,
    )> {
        self.operations
            .iter()
            .map(|i| i.balance_operation())
            .collect::<Vec<_>>()
    }
}

// ----------------------------

#[macro_export]
macro_rules! transaction {
    // https://lukaswirth.dev/tlborm/decl-macros/patterns/push-down-acc.html
    // https://lukaswirth.dev/tlborm/decl-macros/patterns/tt-muncher.html
    ( $( $operations:expr ),+ $(,)? ) => {
        transaction!(; , $( $operations ),+ )
    };
    ( $($accu:expr,)*; , $operation:expr $(, $operations:expr ),* ) => {
        transaction!(
            $($accu,)* Box::new($operation), ;
            $(, $operations ),*
        )
    };
    ( $($accu:expr,)+; ) => {
        MultipleOperationTransaction::new(vec![ $($accu,)+ ])
    };
}
// pub use transaction; // https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files

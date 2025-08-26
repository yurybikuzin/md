use super::*;

pub struct SingleOperationTransaction<T, A>
where
    T: AccountBalanceOperation<A>,
    A: Account,
{
    operation: T,
    marker: PhantomData<A>,
}

impl<T, A> SingleOperationTransaction<T, A>
where
    T: AccountBalanceOperation<A>,
    A: Account,
{
    pub fn new(operation: T) -> Self {
        Self {
            operation,
            marker: PhantomData,
        }
    }
}

impl<T, A> Transaction for SingleOperationTransaction<T, A>
where
    T: AccountBalanceOperation<A>,
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
    T1: AccountBalanceOperation<A>,
    T1: AccountBalanceOperation<A>,
    A: Account,
{
    first_operation: T1,
    second_operation: T2,
    marker: std::marker::PhantomData<A>,
}

impl<T1, T2, A> PairedOperationTransaction<T1, T2, A>
where
    T1: AccountBalanceOperation<A>,
    T2: AccountBalanceOperation<A>,
    A: Account,
{
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
    T1: AccountBalanceOperation<A>,
    T2: AccountBalanceOperation<A>,
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

pub struct MultipleOperationTransaction<A: Account> {
    operations: Vec<Box<dyn AccountBalanceOperation<A>>>,
}

impl<A: Account> MultipleOperationTransaction<A> {
    pub fn new(operations: Vec<Box<dyn AccountBalanceOperation<A>>>) -> Self {
        Self { operations }
    }
}

impl<A: Account> Transaction for MultipleOperationTransaction<A> {
    type TransactionAccount = A;

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
    ( $A:ty => $( $transactions:expr ),+ $(,)? ) => {
        transaction!(@Transactions:
        $A => ;
        $( $transactions ),+ ,)
    };
        (@Transactions: $A:ty => ; $transaction:expr, $($transactions:expr,)* ) => {
            transaction!(@Transactions:
                $A =>
                {
                let boxed: Box<dyn Transaction<TransactionAccount = $A>> = Box::new($transaction);
                boxed
                }
                ;
                $($transactions,)*
            )
        };
        (@Transactions: $A:ty => $accu:block $(+ $accus:block)*; $transaction:expr, $($transactions:expr,)* ) => {
            transaction!(@Transactions:
                $A =>
                $accu $(+ $accus)*
                + {
                    let boxed: Box<dyn Transaction<TransactionAccount = $A>> = Box::new($transaction);
                    boxed
                }
                ;
                $($transactions ,)*
            )
        };
        (@Transactions: $A:ty => $accu:block $(+ $accus:block)*; ) => {
            $accu $(+ $accus)*
        };
    // ----
    ( $( $operations:expr ),+ $(,)? ) => {
        transaction!(@Operations:
            ;
            $($operations),+
            ,
        )
    };
        (@Operations: $($accu:expr,)*; $operation:expr, $($operations:expr,)* ) => {
            transaction!(@Operations:
                $($accu,)* Box::new($operation), ;
                $($operations,)*
            )
        };
        (@Operations: $($accu:expr,)+; ) => {
            MultipleOperationTransaction::new(vec![ $($accu,)+ ])
        };
}
// pub use transaction; // https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files

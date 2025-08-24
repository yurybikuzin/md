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

trait Account<TransactionId: PartialEq + Eq>: Clone {
    type AccountId;
    type LockedAccount: LockedAccount;

    fn id(&self) -> Self::AccountId;
    fn get_locked(
        &self,
        transaction_id: TransactionId,
    ) -> Result<std::sync::Arc<Self::LockedAccount>, AccountError>;
}

trait LockedAccount {
    fn debit(&self, amount: Amount) -> Result<(), AccountError>;
    fn credit(&self, amount: Amount);
}

// ----------------------------

trait Operation<T: Account<TransactionId>, TransactionId: Eq> {
    fn debit_account(&self) -> Option<(&T, Amount)> {
        None
    }
    fn credit_account(&self) -> Option<(&T, Amount)> {
        None
    }
}

// ----------------------------

struct Withdrawal<T: Account<TransactionId>, TransactionId: Eq> {
    account: T,
    debit: Amount,
    marker: std::marker::PhantomData<TransactionId>,
}

impl<T: Account<TransactionId>, TransactionId: Eq> Operation<T, TransactionId>
    for Withdrawal<T, TransactionId>
{
    fn debit_account(&self) -> Option<(&T, Amount)> {
        Some((&self.account, self.debit))
    }
}

// ----------------------------

struct Deposit<T: Account<TransactionId>, TransactionId: PartialEq + Eq> {
    account: T,
    credit: Amount,
    marker: std::marker::PhantomData<TransactionId>,
}

impl<T: Account<TransactionId>, TransactionId: PartialEq + Eq> Operation<T, TransactionId>
    for Deposit<T, TransactionId>
{
    fn credit_account(&self) -> Option<(&T, Amount)> {
        Some((&self.account, self.credit))
    }
}

struct Transfer<T: Account<TransactionId>, TransactionId: PartialEq + Eq> {
    withdrawal: Withdrawal<T, TransactionId>,
    deposit: Deposit<T, TransactionId>,
    marker: std::marker::PhantomData<TransactionId>,
}

impl<T: Account<TransactionId>, TransactionId: PartialEq + Eq> Operation<T, TransactionId>
    for Transfer<T, TransactionId>
{
    fn debit_account(&self) -> Option<(&T, Amount)> {
        self.withdrawal.debit_account()
    }
    fn credit_account(&self) -> Option<(&T, Amount)> {
        self.deposit.credit_account()
    }
}

// ----------------------------

struct TransactionError<T: Account<TransactionId>, TransactionId: PartialEq + Eq> {
    account: T,
    err: AccountError,
    marker: std::marker::PhantomData<TransactionId>,
}

// trait Transaction<T: Account<TransactionId>, TransactionId: std::fmt::Debug> {
trait Transaction {
    type TransactionId: std::fmt::Debug + Eq + Clone;
    type Account: Account<Self::TransactionId>;

    fn id(&self) -> Self::TransactionId;

    fn debit_accounts(&self) -> Vec<(&Self::Account, Amount)>;
    fn credit_accounts(&self) -> Vec<(&Self::Account, Amount)>;

    fn apply(&self) -> Result<(), TransactionError<Self::Account, Self::TransactionId>> {
        let transaction_id = self.id();

        let locked_debit_accounts = {
            let mut locked_debit_accounts = vec![];
            for (account, amount) in self.debit_accounts() {
                match account.get_locked(transaction_id.clone()) {
                    Ok(locked_account) => {
                        locked_debit_accounts.push((account, locked_account, amount));
                    }
                    Err(err) => {
                        return Err(TransactionError {
                            account: account.clone(),
                            err,
                            marker: std::marker::PhantomData,
                        });
                    }
                }
            }
            locked_debit_accounts
        };

        let locked_credit_accounts = {
            let mut locked_credit_accounts = vec![];
            for (account, amount) in self.credit_accounts() {
                match account.get_locked(transaction_id.clone()) {
                    Err(err) => {
                        return Err(TransactionError {
                            account: account.clone(),
                            err,
                            marker: std::marker::PhantomData,
                        });
                    }
                    Ok(locked_account) => {
                        locked_credit_accounts.push((locked_account, amount));
                    }
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
                        marker: std::marker::PhantomData,
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

struct SingleOperationTransaction<T, A, TransactionId>
where
    T: Operation<A, TransactionId>,
    A: Account<TransactionId>,
    TransactionId: std::fmt::Debug + PartialEq + Eq,
{
    id: TransactionId,
    operation: T,
    marker_a: std::marker::PhantomData<A>,
    marker_transaction_id: std::marker::PhantomData<TransactionId>,
}

impl<T, A, TransactionId> Transaction for SingleOperationTransaction<T, A, TransactionId>
where
    T: Operation<A, TransactionId>,
    A: Account<TransactionId>,
    TransactionId: std::fmt::Debug + Eq + Clone,
{
    type TransactionId = TransactionId;
    type Account = A;

    fn id(&self) -> Self::TransactionId {
        self.id()
    }
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

struct PairedOperationTransaction<T1, T2, A, TransactionId>
where
    T1: Operation<A, TransactionId>,
    T1: Operation<A, TransactionId>,
    A: Account<TransactionId>,
    TransactionId: std::fmt::Debug + Eq,
{
    id: TransactionId,
    first_operation: T1,
    second_operation: T2,
    marker_a: std::marker::PhantomData<A>,
    marker_transaction_id: std::marker::PhantomData<TransactionId>,
}

impl<T1, T2, A, TransactionId> Transaction for PairedOperationTransaction<T1, T2, A, TransactionId>
where
    T1: Operation<A, TransactionId>,
    T2: Operation<A, TransactionId>,
    A: Account<TransactionId>,
    TransactionId: std::fmt::Debug + Eq + Clone,
{
    type TransactionId = TransactionId;
    type Account = A;

    fn id(&self) -> Self::TransactionId {
        self.id()
    }
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

struct MultipleOperationTransaction<A, TransactionId>
where
    A: Account<TransactionId>,
    TransactionId: std::fmt::Debug + Eq,
{
    operations: Vec<Box<dyn Operation<A, TransactionId>>>,
    marker_transaction_id: std::marker::PhantomData<TransactionId>,
}

impl<A, TransactionId> Transaction for MultipleOperationTransaction<A, TransactionId>
where
    A: Account<TransactionId>,
    TransactionId: std::fmt::Debug + Eq + Clone,
{
    type TransactionId = TransactionId;
    type Account = A;

    fn id(&self) -> Self::TransactionId {
        self.id()
    }
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

struct LockedSimpleAccount<TransactionId> {
    parent: std::sync::Arc<SimpleAccount<TransactionId>>,
    transaction_id: TransactionId,
}

impl<TransactionId> LockedAccount for LockedSimpleAccount<TransactionId> {
    fn debit(&self, amount: Amount) -> Result<(), AccountError> {
        todo!();
    }
    fn credit(&self, amount: Amount) {
        todo!();
    }
}

struct SimpleAccount<TransactionId> {
    id: u64,
    locked: std::sync::Weak<LockedSimpleAccount<TransactionId>>,
    balance: std::sync::atomic::AtomicU64,
}

impl<TransactionId> LockedAccount for std::sync::Arc<SimpleAccount<TransactionId>> {
    fn debit(&self, amount: Amount) -> Result<(), AccountError> {
        todo!();
    }
    fn credit(&self, amount: Amount) {
        todo!();
    }
}

impl<TransactionId: PartialEq + Eq> Account<TransactionId>
    for std::sync::Arc<SimpleAccount<TransactionId>>
{
    type AccountId = u64;
    type LockedAccount = LockedSimpleAccount<TransactionId>;
    fn id(&self) -> u64 {
        self.id
    }
    fn get_locked(
        &self,
        transaction_id: TransactionId,
    ) -> Result<std::sync::Arc<Self::LockedAccount>, AccountError> {
        if let Some(locked) = self.locked.upgrade() {
            if locked.transaction_id == transaction_id {
                Ok(locked)
            } else {
                Err(AccountError::AlreadyLocked)
            }
        } else {
            let locked = std::sync::Arc::new(LockedSimpleAccount {
                parent: std::sync::Arc::clone(&self),
                transaction_id,
            });
            self.locked = std::sync::Arc::downgrade(&locked);
            Ok(locked)
        }
    }
    // fn unlock(&self) {
    //     self.is_locked
    //         .store(false, std::sync::atomic::Ordering::SeqCst)
    // }
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

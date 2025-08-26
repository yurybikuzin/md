use super::*;

use std::sync::RwLock;

pub struct SimpleAccount<AccountId, Amount>
where
    AccountId: Debug + Clone + Hash + PartialEq + Eq,
    Amount: Debug + Clone + PartialOrd + SubAssign + AddAssign,
{
    id: AccountId,
    balance: std::sync::RwLock<Amount>,
}

impl<AccountId, Amount> Debug for SimpleAccount<AccountId, Amount>
where
    AccountId: Debug + Clone + Hash + PartialEq + Eq,
    Amount: Debug + Clone + PartialOrd + SubAssign + AddAssign,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut binding = f.debug_struct("LockedSimpleAccount");
        binding.field("id", &self.id);
        if let Ok(balance) = self.balance.read() {
            binding.field("balance", &balance);
        }
        binding.finish()
    }
}

impl<AccountId, Amount> SimpleAccount<AccountId, Amount>
where
    AccountId: Debug + Clone + Hash + PartialEq + Eq,
    Amount: Debug + Clone + PartialOrd + SubAssign + AddAssign,
{
    pub fn new(id: AccountId, balance: Amount) -> Arc<Self> {
        Arc::new(Self {
            id,
            balance: RwLock::new(balance),
        })
    }
}

impl<AccountId, Amount> Account for Arc<SimpleAccount<AccountId, Amount>>
where
    AccountId: Debug + Clone + Hash + PartialEq + Eq,
    Amount: Debug + Clone + PartialOrd + SubAssign + AddAssign,
{
    type AccountId = AccountId;
    type LockedAccount<'a>
        = LockedSimpleAccount<'a, Self>
    where
        Amount: 'a,
        AccountId: 'a;
    type Amount = Amount;
    fn id(&self) -> AccountId {
        self.id.clone()
    }
    fn get_locked<'a>(&'a self) -> Result<Self::LockedAccount<'a>, AccountError> {
        if let Ok(guard) = self.balance.write() {
            Ok(LockedSimpleAccount {
                non_commited_balance: guard.clone(),
                guard,
                account: self,
            })
        } else {
            Err(AccountError::Locked)
        }
    }
    fn get_balance(&self) -> Result<Self::Amount, AccountError> {
        self.balance
            .read()
            .map(|guard| guard.clone())
            .map_err(|_| AccountError::Locked)
    }
}

pub struct AmountWrapper<Amount>(Amount)
where
    Amount: Debug + Clone + PartialOrd + SubAssign + AddAssign;

impl<Amount> From<Amount> for AmountWrapper<Amount>
where
    Amount: Debug + Clone + PartialOrd + SubAssign + AddAssign,
{
    fn from(amount: Amount) -> Self {
        Self(amount)
    }
}

impl<AccountId, Amount> Add<AmountWrapper<Amount>> for Arc<SimpleAccount<AccountId, Amount>>
where
    AccountId: Debug + Clone + Hash + PartialEq + Eq,
    Amount: Debug + Clone + PartialOrd + SubAssign + AddAssign,
{
    type Output = Deposit<Self>;

    fn add(self, credit: AmountWrapper<Amount>) -> Self::Output {
        Self::Output::new(self.clone(), credit.0)
    }
}

impl<AccountId, Amount> Sub<AmountWrapper<Amount>> for Arc<SimpleAccount<AccountId, Amount>>
where
    AccountId: Debug + Clone + Hash + PartialEq + Eq,
    Amount: Debug + Clone + PartialOrd + SubAssign + AddAssign,
{
    type Output = Withdrawal<Self>;

    fn sub(self, debit: AmountWrapper<Amount>) -> Self::Output {
        Self::Output::new(self.clone(), debit.0)
    }
}

// ----------------------------

pub struct LockedSimpleAccount<'a, A: Account> {
    account: &'a A,
    non_commited_balance: A::Amount,
    guard: std::sync::RwLockWriteGuard<'a, A::Amount>,
}

impl<'a, A: Account> Debug for LockedSimpleAccount<'a, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockedSimpleAccount")
            .field("account", &self.account)
            .field("non_commited_balance", &self.non_commited_balance)
            .finish()
    }
}

impl<'a, A: Account> LockedAccount<'a, A> for LockedSimpleAccount<'a, A> {
    fn debit(&mut self, amount: A::Amount) -> Result<(), AccountError> {
        if amount > self.non_commited_balance {
            Err(AccountError::InsufficientFunds)
        } else {
            self.non_commited_balance -= amount;
            Ok(())
        }
    }
    fn credit(&mut self, amount: A::Amount) -> Result<(), AccountError> {
        self.non_commited_balance += amount;
        Ok(())
    }
    fn commit(mut self) {
        *self.guard = self.non_commited_balance;
    }
}

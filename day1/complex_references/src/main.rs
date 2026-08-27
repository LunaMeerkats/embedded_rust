// Demonstration on one mutable reference or many 
// immutable references.

fn main() {
    let mut account = BankAccount{
        owner: "Alice".to_string(),
        balance: 150.55,
    };

    account.check_balance();

    account.withdraw(34.50);

    account.check_balance();
}

struct BankAccount {
    owner: String,
    balance: f64,
}

impl BankAccount {
    fn withdraw(&mut self, amount: f64){
        println!("Withdrawing {} from account owned by {}", amount, self.owner);
        self.balance -= amount;
    }

    fn check_balance(&self){
        println!("Account owned by {} has a balance of {}.", self.owner, self.balance);
    }
}

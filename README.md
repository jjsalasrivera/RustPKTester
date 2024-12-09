# Private Key Tester

Do you feel lucky? Try to find a abandoned wallet on Bitcoin blockchain. Keep in mind that there are 10^70 possible addresses; it's more likely to win the lottery than to try to find an abandoned wallet on the blockchain. But who knows?

This program was created with the purpose of learning how Bitcoin addressing works and learn something about parallel programming on Rust.

I have tried to execute the algorithm with tokio and async-std, but I have found that the best performance is with rayon.

## Note

This program use a sqlite database with one table _"Addresses"_ with one collumn _"address"_ which contains all the addresses (indexes) with a balance on the blockchain.

This database is brewed by hand using the csv downloaded from http://addresses.loyce.club/ and is not included in this repository because it's too big, about 6GB.

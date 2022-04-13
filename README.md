# Solana Market Contract

This is a Solana on-chain contract that helps us to trader more clever.
It cooperates with our offline Market Maker.

## Dev Setup

For developing with the smart cotract you'll need the Rust, Solana and Anchor framework at your machine.
Check the most up-to-date information at Anchor site at
https://project-serum.github.io/anchor/getting-started/installation.html#install-rust

Testing is done with Python AnchorPy library. More info can be read at
https://kevinheavey.github.io/anchorpy/testing/


For installation details on Solana and Anchor check the install hint above.
As a shortcut with no confidence of being updated use the following:

* Rust installation, see https://www.rust-lang.org/tools/install
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup component add rustfmt
```
* Solana validator install (`sh -c "$(curl -sSfL https://release.solana.com/v1.9.1/install)"`) or `solana-install update`
* Install Anchor framework
```sh
cargo install --git https://github.com/project-serum/anchor avm --locked --force
avm install latest; avm use latest
```
* Generate solana test key (or check existing one) `solana-keygen pubkey`.\
_Generated publickey is placed at $HOME/.config/solana/id.json_
* Python dependencies can be checked in [requirements.txt](./requirements.txt)

## Working with contract

The contract can be checked by using `make test` that asks the Anchor library
to start the Solana test validator locally, to deploy the contract,
and to run the Python client to check the functionality.


## Deploying contract to validator

### Local

Ledger of the `make test` just runs `anchort test`. The test validator is started automatically.
The data of the validator can be found under `.anchor/test-ledger/`.
Program log can be found under `.anchor/program-logs`.

The test validator can be started manually as well with `solana-test-validator`.
Then we need to say for Anchor to not starting the validator.

```sh
solana-test-validator
solana logs --url localhost
anchor test --skip-deploy --skip-local-validator
```

### Devnet

Working with Mango Markets could be done
[on Devnet](https://github.com/blockworks-foundation/mango-explorer/blob/v.3.4.7/data/ids.json#L472).

#### Getting SOLs from faucet

Faucet to get Solana for deployment is at https://solfaucet.com/ or `solana airdrop -u devnet 2` (cap for airdrop is 2).
Transaction confirmation (or error) can be observed with
`solana confirm -v jiNRKyR8dGJy5DMbc49jbhZsFMYmF71jYy62iS1YvoRF1WMXfUdzEpfxc4s48UyhyGRu6XMmaapYCZx5Va2CL9X -u devnet`.

#### Anchor deploy to Devnet and test

```sh
make build
# or
anchor build

# anchor creates folder target/deploy
# there is the output .so file and keypair that will be used
# for deployment of the program - it's the program id
solana address -k target/deploy/market_contract-keypair.json

# !!! ACTION ITEM !!!
# you will probably need to change the Anchor.toml and lib.rs declare_id! and the client python program
# to contain the same public key that is generated for deployment

anchor deploy --provider.cluster=devnet
anchor deploy --provider.cluster='https://mango.devnet.rpcpool.com'

# deployment via solana cli
# to show existing programs for the wallet
solana program show --programs -u devnet
solana program show --buffers -u devnet


solana program deploy -u devnet \
  --program-id target/deploy/market_contract-keypair.json target/deploy/market_contract.so
solana program close BF8mmQDfXK6FVLKCwtcJ48R5J9fQbdBatBRpsieaQV7h -u devnet
```

For checking the transaction it can be used the `solana confirm` or the explorer on devnet cluster:
https://explorer.solana.com/?cluster=devnet

**WARN**:  be prepared that the first(**!**) deployment eats about 4SOL from your wallet.
The deployment creates two accounts - the executable account + data account with Rust binaries.
Depending on the size of the Rust binaries you need to place a rent to the data account
otherwise the Solana runtime removes the account out of the cluster.
SOLs will be deposited back to your wallet when the Solana accounts are cleared.

The folder `client` contains a test Python program that could be adjusted for particular test purposes.

```sh
python client/client-test.py
```

To work with Mango on devnet you can use the Python cli provided by Mango Python library
https://github.com/blockworks-foundation/mango-explorer/

```sh
git clone https://github.com/blockworks-foundation/mango-explorer/
cd mango-explorer/bin

# for the correct name of the cluster check
cat data/ids.json

# check if mango account exists or create it when it does not exist (call it twice to create and show)
./ensure-account --cluster-name devnet --id-file ~/.config/solana/id.json

# check if mango associated account to the token exists
# if not then it's created (account rent-excepmt reserve has to be paid ~0.002) [1]
./bin/ensure-associated-token-account --cluster-name devnet --id-file ~/.config/solana/id.json --symbol SOL
./bin/ensure-associated-token-account --cluster-name devnet --id-file ~/.config/solana/id.json --symbol USDC
./bin/show-all-token-accounts --cluster-name devnet --id-file ~/.config/solana/id.json

# for sending SOL from my account to mango account the sol needs to be wrapped
./bin/wrap-sol --cluster-name devnet --id-file ~/.config/solana/id.json  --quantity 1
./bin/deposit --cluster-name devnet --id-file ~/.config/solana/id.json --symbol SOL --quantity 1
./bin/show-account-balances --cluster-name devnet --id-file ~/.config/solana/id.json

# let's place an order
./bin/place-order --cluster-name devnet --id-file ~/.config/solana/id.json --market SOL-PERP --quantity 0.5 --price 103 --side BUY --order-type LIMIT
./bin/show-orderbook --cluster-name devnet --id-file ~/.config/solana/id.json --market SOL-PERP

# let's cancel it
./bin/show-my-orders --cluster-name devnet --id-file ~/.config/solana/id.json --market SOL-PERP
./bin/cancel-order --cluster-name devnet --id-file ~/.config/solana/id.json --market SOL-PERP --client-id 1649750402198
```

* __[1]__: output

```
No associated token account at: C6UvPKBiDvHXwMFAx56PYKRuwHJZd33wY6b5hZ8wkyw7 - creating...
Transaction signatures:
    B4VNCtEL6hLUhu5sBCPef8nf9gnPnpsGfDVTPdAGgRhMYdyFoxQAPzxoxR1nur1bQaoj5HgjJtgFKLX4gEpC8gn
Associated token account created at: C6UvPKBiDvHXwMFAx56PYKRuwHJZd33wY6b5hZ8wkyw7.
```


## Troubleshooting

### Errors

Some errors that was hit during development and could help in future.

__NOTE:__ some errors can be checked at:
  https://solongwallet.medium.com/solana-development-tutorial-things-you-should-know-before-structuring-your-code-807f0e2ee43

#### 1)

```
E           solana.rpc.core.RPCException: {'code': -32002, 'message': 'Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account', 'data': {'accounts': None, 'err': {'InstructionError': [0, 'PrivilegeEscalation']}, 'logs': ['Program HpVi2Nyw9SxD9UssvzqYMpG71jnQFaQXjfFpDt6gsf3R invoke [1]', 'Program log: Instruction: Create', "FnVL2AATb8P22Ktip1oYB5oGAtReio5VkNsEPmNXoJsS's signer privilege escalated", 'Program HpVi2Nyw9SxD9UssvzqYMpG71jnQFaQXjfFpDt6gsf3R consumed 11510 of 1400000 compute units', 'Program HpVi2Nyw9SxD9UssvzqYMpG71jnQFaQXjfFpDt6gsf3R failed: Cross-program invocation with unauthorized signer or writable account'], 'unitsConsumed': 0}}
```

Errors thrown on creating PDA account. It seems it could be caused by fact the transaction is wrongly signed,
StackOverflow talks about possibility that there is running a Solana test validator (it needs to be stopped
before the `anchor test` is launched as anchor starts own test validator before the test starts).
The issue that was hit here on development was fact that the seed generated by Python was not correct
and the PDA address was then probably hitting the Solana standard adress which is not possible for PDA.

#### 2)

```
E           solana.rpc.core.RPCException: {'code': -32002, 'message': 'Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1004', 'data': {'accounts': None, 'err': {'InstructionError': [0, {'Custom': 4100}]}, 'logs': ['Program BF8mmQDfXK6FVLKCwtcJ48R5J9fQbdBatBRpsieaQV7h invoke [1]', 'Program log: AnchorError occurred. Error Code: DeclaredProgramIdMismatch. Error Number: 4100. Error Message: The declared program id does not match the actual program id.', 'Program BF8mmQDfXK6FVLKCwtcJ48R5J9fQbdBatBRpsieaQV7h consumed 3641 of 1400000 compute units', 'Program BF8mmQDfXK6FVLKCwtcJ48R5J9fQbdBatBRpsieaQV7h failed: custom program error: 0x1004'], 'unitsConsumed': 0}}
```

For example when `make test` is run. This could be caused by changes in `Anchor.toml` and in `lib.rs` `declare_id` macro.
When changes happen it could be trouble that local data test validator is not updated.

First verify that you do not run `solana-test-validator` somewhere around.
Then it could help to delete `target` directory and the `.anchor` test ledger.

```sh
rm -rf .anchor/ target/
```

#### 3)

```
solana.rpc.core.RPCException: {'code': -32002, 'message': 'Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account'
...
"CNg4H4VTdyMWysHG3QhepqXYTZima1g1wSwfzaLHRFmh's writable privilege escalated", 'Program B8hvuv3LXchAe4Wm5EVAKUntUsymGyAh1n8dfM5KuR3d consumed 114716 of 200000 compute units', 'Program B8hvuv3LXchAe4Wm5EVAKUntUsymGyAh1n8dfM5KuR3d failed: Cross-program invocation with unauthorized signer or writable account'], 'unitsConsumed': 0}}
```

This means that the account configuration does not set
mutability with `#[account(mut)]` at account declaration.


#### 4)

```
Program failed to complete: Access violation in stack frame 7 at address 0x200007ff8 of size 8 by instruction #1104
```

This means probably the memory consumption is over permitted numbers for stack.
It could help to use the `Box` at account declaration to create the `Account`
at heap and not at stack. Try to do like `Box<Account<'info, Whatever>>`.
Then it's possible to use `.to_account_info()` later,
since Account is a wrapper around `AccountInfo`.

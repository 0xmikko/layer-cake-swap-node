![title](https://user-images.githubusercontent.com/26343374/106135940-a01b1a80-6179-11eb-956c-e6139f634973.png)
## LayerCakeSwap

LayerCakeSwap is a DEX protocol which consumes 7 times less gas<sup>1</sup> in comparison with UniSwap, based on L2 network. It solves 
the problem of 

- High gas rates make DeFi expensive for users  ~$12-18 USD per swap
- Users prefer and trust Ethereum and aren’t willing to use side chains solutions (like Binance Smart Chain, POA, etc.) for DeFi

Users interact with LayerCakeSwap via familiar Ethereum interface with no additional setup. Such gas efficiency comes from 
movement all calculations & storage operations to L2 networks (Polkadot, Substrate).

![advanages](https://user-images.githubusercontent.com/26343374/106137793-4405c580-617c-11eb-8b6c-59190649bbe8.png)

This version implements business logic for trading a pair ETH - ERC20 Token. Ethereum contract & frontend is located at https://github.com/MikaelLazarev/layer-cake-swap-client/

This project was developed for Encode Hackathon'2021. 

### Gas consumption comparison
| Operation                    | UniSwap     | LayerCakeSwap |
|------------------------------|:-----------:|:-------------:|
| Deposit Eth                  | -           |  22,656       |
| Deposit token                | -           |  48,628       | 
| Swap token to Eth<sup>2</sup>| 165,969     |  22,656       |
| AddToLiquidity pool          | in research |  22,656       |
| Withdraw from liquidity pool | in research |  22,656       |
| Withdraw ETH                 | -           |  no data yet  |
| Withdraw token               | -           |  no data yet  |

<sup>[1](#myfootnote1)</sup> - for swap operation. UniSwap gas consumption was measured for ETH/DAI pair.  
<sup>[2](#myfootnote1)</sup> - gas for UniSwap was measured for ETH/DAI Swap

## How it works

![how_it_works](https://user-images.githubusercontent.com/26343374/106125934-cf2b8f00-616d-11eb-8874-2ae3d08ccf6b.png)

1. User interacts with smart contract on Ethereum network as usual
2. Contract processes method and emits an event
3. LCSwap listens to events and execute orders on L2

### Supported operations:

Deposit assets (Ethereum / Token)

![deposit_process](https://user-images.githubusercontent.com/26343374/106126399-5f69d400-616e-11eb-9d63-7e2360e5da49.png)

Orders (Swap, Liquidity pool management)

![order_process](https://user-images.githubusercontent.com/26343374/106126526-845e4700-616e-11eb-89be-752d8ea2f472.png)

Withdraw process

![withdraw_process](https://user-images.githubusercontent.com/26343374/106127954-08fd9500-6170-11eb-891a-550223ceb0b3.png)

Legend

![legend](https://user-images.githubusercontent.com/26343374/106128944-e15afc80-6170-11eb-9880-5fc25f9e3fe6.png)

## How to install (dev mode)

This project contains some configuration files to help get started :hammer_and_wrench:

### Rust Setup

Setup instructions for working with the [Rust](https://www.rust-lang.org/) programming language can
be found at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started). Follow those
steps to install [`rustup`](https://rustup.rs/) and configure the Rust toolchain to default to the
latest stable version.

### Clone contract repo & deploy ethereum smartcontract

1. Clone contracts repo: `git clone https://github.com/MikaelLazarev/layer-cake-swap-client/`
2. Run hardhat blockchain: `npx hardhat node`. Do not close this tab, hardhat node should work in background
3. Deploy smartcontract: `yarn deploy-local`
4. Run frontend: `yarn start`

### Clone Substrate node repo & config it

1. Clone this repo: `git clone git@github.com:MikaelLazarev/layer-cake-swap-node.git`
2. Go to `src` folder in your frontend folder and open `env.local` file.
3. It would contain something like that:
```
REACT_APP_BACKEND_ADDR=http://localhost:8000
REACT_APP_VAULT_ADDRESS=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
REACT_APP_TOKEN_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3
REACT_APP_CHAIN_ID=1337
```
4. Copy `REACT_APP_VAULT_ADDRESS` and `REACT_APP_TOKEN_ADDRESS`, we will need them to configure substrate node.
5. Go back to Substrate node directory
6. Copy `.env_example` to `.env` file and set web3 provider. For dev pruposes it would be hardhat address:
```
WEB3PROVIDER=http://localhost:8545
```
7. Go to `polkaswap` pallet directory (it's former name of layer-cake-swap project): `cd pallets/polkaswap/src`
8. Open `lib.rs` file and insert vault contract address & token address there withot `0x` prefix:
```rust
/// Vault contract address
pub const VAULT_CONTRACT_ADDRESS: &'static str = "e7f1725E7734CE288F8367e1Bb143E90bb3F0512";

/// Token contract agaist Eth erc20 contract address
/// DAI on our case
pub const TOKEN_CONTRACT_ADDRESS: &'static str = "5FbDB2315678afecb367f032d93F642f64180aa3";
```
9. Return to root directory and run substrate node in dev mode: `make dev`

## 

## Disclaimer

This application is provided "as is" and "with all faults." Me as developer makes no representations or 
warranties of any kind concerning the safety, suitability, lack of viruses, inaccuracies, typographical 
errors, or other harmful components of this software. There are inherent dangers in the use of any software, 
and you are solely responsible for determining whether this software product is compatible with your equipment and 
other software installed on your equipment. You are also solely responsible for the protection of your equipment 
and backup of your data, and THE PROVIDER will not be liable for any damages you may suffer in connection with using, 
modifying, or distributing this software product.

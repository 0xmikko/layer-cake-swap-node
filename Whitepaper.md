![title_white](https://user-images.githubusercontent.com/26343374/106140486-e5dae180-617f-11eb-86e9-cb4204589d6c.png)
## LayerCakeSwap

LayerCakeSwap is a DEX protocol which consumes 7 times less gas<sup>1</sup> in comparison with UniSwap, based on L2 network.
It solves following problems: 

- High gas rates make DeFi expensive for users  ~$12-18 USD per swap (@ 1,000 USD/Eth)
- DeFi users prefer and trust Ethereum and aren’t willing to migrate to side chains solutions (like Binance Smart Chain, POA, etc.) for gas efficiency

Users interact with LayerCakeSwap smart contract on Ethereum using familiar interface, no additional setup required. 
Even more, LayerCakeSwap contract API is accessible for other smart contract on Ethereum. 

Smart contract emits events to manage operations on L2 network based on Polkadot (Substrate). Gas efficiency comes from executing calculations & 
data storage on L2 network.

![advanages](https://user-images.githubusercontent.com/26343374/106137793-4405c580-617c-11eb-8b6c-59190649bbe8.png)


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

## 

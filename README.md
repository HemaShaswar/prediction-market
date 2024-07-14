# Talent Olympics 2024 Prediction Market & Blink for Memecoins Project

## Introduction

Welcome to the Talent Olympics 2024 Prediction Market & Blink for Memecoins Project! This decentralized application (DApp) allows users to participate in prediction markets for the Talent Olympics 2024 and engage in Blink for Memecoins activities. Users can create markets, place bets on various outcomes, and claim their rewards based on real-time price feeds from the Pyth Network.

## Features

- **Market Initialization:** Create prediction markets with specific target prices and durations.
- **Dynamic Odds Calculation:** Place bets with dynamically calculated odds based on the current state of the market.
- **Real-Time Price Feeds:** Integrate real-time price data from the Pyth Network.

## Dependencies

```toml
[dependencies]
anchor-lang = "0.30.1"
anchor-spl = "0.30.1"
num-traits = "0.2.19"
num-derive = "0.4.2"
pyth-solana-receiver-sdk = "0.2.0"
```

### Installation Steps

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/your-username/talent-olympics-2024.git
   cd talent-olympics-2024
   ```

2. **Build the Solana Program:**

   ```bash
   anchor build
   ```

3. **Deploy the Solana Program:**
   ```bash
   anchor deploy
   ```

## Challenges and Improvements

**Automated Task Execution:**

- Using the Clockwork SDK to automate tasks such as market closure and price fetching introduced complexities in ensuring reliability and security.

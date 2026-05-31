# Backtesting
-  Why backtest? Past performance is not an indicator of future performance
>A failing backtest almost guarantees a failing future. If your strategy cannot even make money in a historical environment perfectly suited for it, it has zero chance of surviving live markets. It saves you from risking real capital on fundamentally flawed logic.
> 
> Markets change, but human psychology and structural mechanics do not. Even if the exact sequence of prices never repeats, the statistical characteristics of market panics, liquidations, and trends do.

## Backtesting -> Real investing pipeline

```mermaid
graph TD
A["Historical Backtest: Validates basic logic & risk boundaries"] --> B[Out-of-Sample Testing]
B["Out-of-Sample Testing: Tests the strategy on data it has never seen before"] --> C[Paper Trading / Forward Testing]
C["Paper Trading / Forward Testing: Simulates live market execution, latency, and slippage"] --> D[Live Production Small Capital]
```

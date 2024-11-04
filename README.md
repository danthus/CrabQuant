# **Performant Software Systems with Rust**

## **ECE 1724**

## **Project Proposal**

**Hanzhen Xu 1004475285**  
**Haoran Zhou 1003970713**  
**Yue Chen 1010813675**

## **Motivation**

Accurate, efficient, and reproducible backtesting is fundamental to the research and development of quantitative trading strategies. As the critical first step in validating a strategy's effectiveness, backtesting allows traders to simulate performance on historical data, offering essential insights before any capital is put at risk in live markets.

Currently, backtesting frameworks are predominantly Python-based. Python offers advantages in accessibility, ease of use, and a wide range of libraries suited to financial data analysis and model building. For lower-frequency strategies where data volumes are manageable, Python is an ideal tool. However, for higher-frequency strategies, data volumes increase significantly, and it may take several hours or even overnight to complete a single backtest, even for relatively simple strategies. These performance limitations not only slow down the work of individual quantitative analysts, but many quantitative funds also need to maintain separate Python-based backtesting and research environments alongside their actual algorithmic trading deployments in higher-performance languages. This separation introduces additional complexity, uncertainty, and potential inconsistencies between the two environments.

In this project, we aim to develop a foundational backtesting framework using Rust. This framework will facilitate more efficient backtesting on large high-frequency datasets, enabling the later development of “live trading” modules that promise consistency between testing and trading environments, and benefit from Rust’s efficiency and reliability. The performance and memory safety characteristics of Rust language offer distinct advantages for high-frequency and data-intensive applications, potentially enabling a unified model for both backtesting and live trading in a high-performance setting.

## **Objective and Key Features**

In general, our objective is to design and build a new rust-based backtesting framework, assisting users to test their own trading strategies on corresponding dataset. It should also offer a basic visualization tool for a descriptive and comparable view on return and risk of their promoted strategies. An example output would be like the following figure (produced with Python).  

![Alt text][image1]  
Fig 1, Sample backtesting result plot.

Specifically, the primary objective of the project is modularity. While Rust offers the greatest potential for enhancing an algo-trading framework in both performance and reliability, our focus remains on modularity due to limited resources and the fact that Rust is new to all team members. By prioritizing modularity as our design principle, we aim for the utmost decoupling of modules, allowing the framework to be adaptable for future upgrades—such as asynchronous support, concurrency, and data serializing/deserializing—and enabling users to focus on developing trading strategies. Guided by this principle, we chose an event-driven architecture in which each module functions like a microservice, communicating solely through events. This architecture not only achieves the modularity we aim for but also frees users from dealing with the implementation details of other modules.

Some of the key features of our backtesting framework include the following:

1. Event-driven architecture.  
2. Future data prevention.  
3. Customizable fees and slippage settings.  
4. Flexible time frame selection, from ticks to daily bars.  
5. Customizable backtest time window.  
6. Visual output/report of backtesting results, and comparison to baseline.  
7. Assessment on return and risk on strategy.  
8. Detailed logging.

## **Tentative Plan**

Our team has completed our initial event-driven architecture design as demonstrated by the following diagram. The control-flow will be handled by an EventManager, which maintains a queue for all event types. Each time a current event has been completely handled, the EventManager will dispatch the next event to all modules that subscribe to it. In the following diagram, we demonstrate how modules subscribe to types of events, in which a square represents a type of event, a diamond represents a module, double-arrows represent publish events to the queue, single-arrows represent subscription relations, and circles represent static-lifetime items.  
![Alt text][image2]  
The workload of the project is then divided into the following components, by how each is naturally decoupled with others in the architecture. Their corresponding features are also listed in the table.

* EventManager: Feature A  
* MockExchange: Feature B, C  
* SampleStrategyModule: Feature D, E  
* PortfolioManager  
* MarketDataFeeder  
* AnalyticEngine: Feature F, G and H.

At the time of writing this report, our team already found proper and authentic datasets for backtesting and revised the framework multiple times in detail. What is left to finish is the coding and testing parts. To finish the these parts in one month, team members would divide the coding and testing workloads by modules as following:  
Member A: EventManager and MockExchange  
Member B: PortfolioManager, Sample StrategyModule, and MarketDataFeeder  
Member C: AnalyticEngine, Visualization

For each finished module, the corresponding member will also run unit tests on it individually. In this way, the unit tests can be more specific to each module and straight-forward compared to complex tests involving multiple modules. Then, the tested module will be integrated into the framework. Due to the decoupling nature of the framework, the integration will be efficient and time consumption will be minimal.  
Additionally, our team will also arrange weekly in-person meetings to check members’ progress, discuss current problems and make sure the team is on the right track.  

[image1]:resources/image/MACROSS_5_20_sample.png
[image2]:resources/image/CrabQuantArchitecture.png
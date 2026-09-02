import { graphql } from "../gql/gql";

export const GET_MAIN_SITE_DATA = graphql(/* GraphQL */ `
  query GetMainSiteData {
    apiCounts
    budget {
      currentFunds
      ironReserve
      reservedAmount
      spendable
    }
    runInfo {
      resetDate
      nextResetDate
      agent {
        symbol
        credits
        shipCount
      }
      headquartersSystem {
        symbol
        constructionMaterials {
          items {
            waypointSymbol
            tradeSymbol
            required
            fulfilled
          }
        }
      }
    }
    systems(onlyWithFleetsOrShips: true) {
      items {
        symbol
        waypoints {
          items {
            symbol
            chartedBy
            hasMarketplace
            hasShipyard
          }
        }
      }
    }
    fleets {
      items {
        id
        systemSymbol
        fleetType
        active
        assignments {
          items {
            id
            priority
            rangeMin
            cargoMin
            ship {
              symbol
            }
          }
        }
      }
    }
    shipAssignments(by: { open: true }) {
      items {
        id
        fleetId
        fleet {
          systemSymbol
          fleetType
        }
      }
    }
    ships {
      symbol
      registrationRole
      status {
        assignmentId
        tempAssignmentId
        status {
          __typename
        }
      }
      nav {
        status
        systemSymbol
      }
      cargo {
        units
      }
      cooldownExpiration
    }
    chartManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    fleetManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    shipProcurementManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    tradeManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    miningManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    contractManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    scrappingManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
    constructionManager {
      busy
      channelState {
        usedCapacity
        state
      }
    }
  }
`);

export const GET_ALL_SYSTEMS = graphql(/* GraphQL */ `
  query GetAllSystems {
    systems {
      items {
        symbol
        constellation
        sectorSymbol
        systemType
        x
        y
        populationDisabled
        waypoints {
          items {
            symbol
            waypointType
            hasShipyard
            hasMarketplace
          }
        }
        fleets {
          items {
            id
            fleetType
            active
          }
        }
        ships {
          symbol
        }
      }
    }
  }
`);

export const GET_SYSTEM_MAP_DATA = graphql(/* GraphQL */ `
  query GetSystemMapData {
    systems {
      items {
        symbol
        constellation
        systemType
        x
        y
        populationDisabled
        waypoints {
          items {
            symbol
            waypointType
            hasShipyard
            hasMarketplace
            isUnderConstruction
          }
        }
        fleets {
          items {
            id
            fleetType
            active
          }
        }
        ships {
          symbol
        }
      }
    }
    jumpConnections {
      items {
        underConstructionA
        underConstructionB
        pointASymbol
        pointBSymbol
        fromA
        fromB
      }
    }
  }
`);

export const GET_ALL_AGENTS = graphql(/* GraphQL */ `
  query GetAllAgents {
    agents {
      symbol
      credits
      shipCount
      startingFaction
      headquarters
      createdAt
    }
  }
`);

export const GET_AGENT_HISTORY = graphql(/* GraphQL */ `
  query GetAgentHistory($agentSymbol: String!) {
    agent(symbol: $agentSymbol) {
      symbol
      credits
      shipCount
      accountId
      startingFaction
      createdAt
      headquarters
      history {
        id
        credits
        shipCount
        createdAt
      }
    }
  }
`);

export const GET_ALL_SURVEYS = graphql(/* GraphQL */ `
  query GetAllSurveys {
    surveys {
      items {
        shipInfoBefore
        updatedAt
        shipInfoAfter
        signature
        signature
        size
        waypointSymbol
        deposits
        exhaustedSince
        createdAt
        expiration
      }
    }
  }
`);

export const GET_SYSTEM = graphql(/* GraphQL */ `
  query GetSystem($systemSymbol: String!) {
    system(symbol: $systemSymbol) {
      symbol
      sectorSymbol
      constellation
      systemType
      x
      y
      populationDisabled
      seenAgents {
        symbol
        count
      }
      fleets {
        items {
          id
          fleetType
          active
          assignments {
            items {
              id
              siphon
              warpDrive
              fleetId
              priority
              maxPurchasePrice
              creditsThreshold
              disabled
              rangeMin
              cargoMin
              survey
              extractor
              ship {
                symbol
              }
            }
          }
          config {
            __typename
            ... on TradingConfig {
              tradeMode
            }
            ... on ChartingConfig {
              chartOnlyJumpGates
            }
          }
          createdAt
          updatedAt
        }
      }
      chartTransactions {
        items {
          waypointSymbol
          shipSymbol
          totalPrice
          timestamp
        }
      }
      shipyardShips {
        items {
          reactorQuality
          engineType
          engineQuality
          modules
          mounts
          createdAt
          waypointSymbol
          shipType
          name
          supply
          activity
          purchasePrice
          frameType
          frameQuality
          reactorType
        }
      }
      marketTrades {
        items {
          waypointSymbol
          symbol
          createdAt
          type
          tradeSymbolInfo {
            symbol
            requires {
              items {
                symbol
              }
            }
            requiredBy {
              items {
                symbol
              }
            }
          }
          marketTradeGood {
            symbol
            waypointSymbol
            type
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
            createdAt
          }
        }
      }
      constructionMaterials {
        items {
          waypointSymbol
          tradeSymbol
          required
          fulfilled
          updatedAt
          marketTransactionSummary {
            allExpenses
            allPurchaseUnits
            allPurchaseTransactions
          }
        }
      }
      jumpGateConnections {
        items {
          from
          to
        }
      }
      waypoints {
        items {
          symbol
          faction
          modifiers
          chartedBy
          chartedOn
          hasShipyard
          hasMarketplace
          x
          y
          lastScrap
          nextScrap
          waypointType
          traits
          isUnderConstruction
          orbitals
          orbits
          marketTrades {
            items {
              symbol
              type
              tradeSymbolInfo {
                symbol
                requires {
                  items {
                    symbol
                  }
                }
                requiredBy {
                  items {
                    symbol
                  }
                }
              }
              marketTradeGood {
                tradeVolume
                supply
                activity
                purchasePrice
                sellPrice
              }
            }
          }
          shipyardShips {
            items {
              shipType
              supply
              activity
              purchasePrice
            }
          }
        }
      }
      shipyardTransactions {
        items {
          id
          waypointSymbol
          shipType
          price
          agentSymbol
          timestamp
        }
      }
      contractDeliveries {
        items {
          contractId
          tradeSymbol
          destinationSymbol
          unitsRequired
          unitsFulfilled
          contract {
            id
            createdAt
            factionSymbol
            contractType
            accepted
            onFulfilled
            onAccepted
            deadline
            marketTransactionSummary {
              allExpenses
            }
          }
        }
      }
      tradeRoutes {
        items {
          id
          marketTransactionSummary {
            allExpenses
            allIncome
          }
          symbol
          shipSymbol
          PurchaseWaypointSymbol
          SellWaypointSymbol
          tradeMode
          purchaseMarketTradeGood {
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
          }
          sellMarketTradeGood {
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
          }
          estimatedFuel
          status
          tradeVolume
          createdAt
        }
      }
      ships {
        symbol
        nav {
          waypointSymbol
          status
        }
        fuel {
          capacity
        }
        cargo {
          capacity
        }
        status {
          assignmentId
          fleetId
          tempAssignmentId
          tempFleetId
          status {
            __typename
          }
        }
      }
    }
  }
`);

export const GET_SYSTEM_MARKETS = graphql(/* GraphQL */ `
  query GetSystemMarkets($systemSymbol: String!) {
    system(symbol: $systemSymbol) {
      symbol
      fleets {
        items {
          id
          fleetType
          active
          assignments {
            items {
              id
              siphon
              warpDrive
              fleetId
              priority
              maxPurchasePrice
              creditsThreshold
              disabled
              rangeMin
              cargoMin
              survey
              extractor
            }
          }
          config {
            __typename
            ... on TradingConfig {
              tradeMode
              marketBlacklist
              marketPreferList
              purchaseMultiplier
              tradeProfitThreshold
              shipMarketRatio
              minCargoSpace
            }
          }
          createdAt
          updatedAt
        }
      }
      ships {
        symbol
        nav {
          waypointSymbol
          status
        }
        fuel {
          capacity
        }
        cargo {
          capacity
        }
        engineSpeed
        status {
          assignmentId
          fleetId
          tempAssignmentId
          tempFleetId
          status {
            __typename
          }
        }
      }
      marketTrades {
        items {
          waypointSymbol
          symbol
          createdAt
          type
          tradeSymbolInfo {
            symbol
            requires {
              items {
                symbol
              }
            }
            requiredBy {
              items {
                symbol
              }
            }
          }
          marketTradeGood {
            symbol
            waypointSymbol
            type
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
            createdAt
          }
        }
      }
      tradeRoutes {
        items {
          id
          marketTransactionSummary {
            allExpenses
            allIncome
          }
          symbol
          shipSymbol
          PurchaseWaypointSymbol
          SellWaypointSymbol
          tradeMode
          purchaseMarketTradeGood {
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
          }
          sellMarketTradeGood {
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
          }
          estimatedFuel
          status
          tradeVolume
          createdAt
        }
      }
    }
  }
`);

export const GET_SYSTEM_TRADE_ROUTE_CANDIDATES = graphql(/* GraphQL */ `
  query GetSystemTradeRouteCandidates(
    $systemSymbol: String!
    $source: ShipNavStatsSource
    $purchaseMultiplier: Float
  ) {
    system(symbol: $systemSymbol) {
      symbol
      tradeRouteCandidates {
        items {
          symbol
          purchase {
            symbol
            waypointSymbol
            type
          }
          sell {
            symbol
            waypointSymbol
            type
          }
          tradeRouteProposal(
            source: $source
            purchaseMultiplier: $purchaseMultiplier
          ) {
            symbol
            travelCost
            goodCost
            totalCost
            goodTotalSellPrice
            goodProfit
            totalProfit
            tripsPerHour
            profitPerHour
            profitPerApiRequest
            fuelUnits
            time
            distance
            apiRequests
            tradeVolume
            purchaseGood {
              tradeVolume
              supply
              activity
              purchasePrice
              sellPrice
            }
            sellGood {
              tradeVolume
              supply
              activity
              purchasePrice
              sellPrice
            }
          }
        }
      }
    }
  }
`);

export const GET_SYSTEM_MAP = graphql(/* GraphQL */ `
  query GetSystemMap($systemSymbol: String!) {
    system(symbol: $systemSymbol) {
      symbol
      sectorSymbol
      constellation
      systemType
      x
      y
      populationDisabled
      waypoints {
        items {
          symbol
          faction
          modifiers
          chartedBy
          chartedOn
          hasShipyard
          hasMarketplace
          x
          y
          lastScrap
          nextScrap
          waypointType
          traits
          isUnderConstruction
          orbitals
          orbits
          marketTrades {
            items {
              symbol
              type
              tradeSymbolInfo {
                symbol
                requires {
                  items {
                    symbol
                  }
                }
                requiredBy {
                  items {
                    symbol
                  }
                }
              }
              marketTradeGood {
                tradeVolume
                supply
                activity
                purchasePrice
                sellPrice
              }
            }
          }
          shipyardShips {
            items {
              shipType
              supply
              activity
              purchasePrice
            }
          }
        }
      }
      ships {
        symbol
        status {
          waitingForManager
          waitingForApi
          status {
            __typename
            ... on ScraperStatus {
              scrapDate
            }
          }
        }
        nav {
          waypointSymbol
          status
          flightMode
          route {
            destinationSymbol
            originSymbol
            departureTime
            arrival
          }
          autoPilot {
            destinationSymbol
            originSymbol
            route {
              connections {
                ... on NavigateConnection {
                  startSymbol
                  endSymbol
                  navMode
                }
              }
            }
          }
        }
      }
    }
  }
`);

export const GET_WAYPOINT = graphql(/* GraphQL */ `
  query GetWaypoint($waypointSymbol: String!) {
    waypoint(symbol: $waypointSymbol) {
      symbol
      systemSymbol
      waypointType
      traits
      faction
      chartedBy
      chartedOn
      unstableSince
      x
      y
      lastScrap
      nextScrap
      orbitals
      orbits
      isUnderConstruction
      hasMarketplace
      hasShipyard
      modifiers
      jumpGateConnections {
        items {
          id
          from
          to
        }
      }
      marketTrades {
        items {
          symbol
          type
          tradeSymbolInfo {
            symbol
            requires {
              items {
                symbol
              }
            }
            requiredBy {
              items {
                symbol
              }
            }
          }
        }
      }
      marketTradeGoods {
        items {
          symbol
          tradeVolume
          supply
          activity
          purchasePrice
          sellPrice
          type
          createdAt
        }
      }
      constructionMaterials {
        items {
          id
          tradeSymbol
          required
          fulfilled
          updatedAt
          marketTransactionSummary {
            allExpenses
            allPurchaseUnits
            allPurchaseTransactions
          }
        }
      }
      marketTransactions {
        items {
          id
          waypointSymbol
          shipSymbol
          tradeSymbol
          type
          units
          pricePerUnit
          totalPrice
          timestamp
          contract_id
          trade_route_id
          mining_waypoint_symbol
          construction_shipment_id
          isFuel
        }
      }
      shipyard {
        waypointSymbol
        modificationsFee
        createdAt
      }
      shipyardShipTypes {
        items {
          id
          shipType
          createdAt
        }
      }
      shipyardShips {
        items {
          shipType
          supply
          activity
          purchasePrice
          createdAt
        }
      }
      shipyardTransactions {
        items {
          id
          shipType
          price
          agentSymbol
          timestamp
        }
      }
    }
  }
`);

export const GET_WAYPOINT_HISTORY = graphql(/* GraphQL */ `
  query GetWaypointHistory($waypointSymbol: String!) {
    waypoint(symbol: $waypointSymbol) {
      symbol
      marketTrades {
        items {
          symbol
          type
          tradeSymbolInfo {
            symbol
            requires {
              items {
                symbol
              }
            }
            requiredBy {
              items {
                symbol
              }
            }
          }
          marketTradeGood {
            history {
              items {
                createdAt
                purchasePrice
                sellPrice
                tradeVolume
                supply
                activity
              }
            }
          }
        }
      }
    }
  }
`);

export const GET_CHART_TRANSACTIONS = graphql(/* GraphQL */ `
  query GetChartTransactions {
    chartTransactions {
      items {
        waypointSymbol
        shipSymbol
        totalPrice
        timestamp
        waypoint {
          symbol
          waypointType
          traits
        }
      }
    }
  }
`);

export const GET_API_COUNT = graphql(/* GraphQL */ `
  query GetApiCount {
    apiCounts
  }
`);

export const GET_MARKET_TRANSACTIONS = graphql(/* GraphQL */ `
  query GetMarketTransactions($page: Int, $pageSize: Int) {
    marketTransactions(page: $page, pageSize: $pageSize) {
      page
      pageSize
      totalCount
      items {
        id
        waypointSymbol
        shipSymbol
        tradeSymbol
        type
        units
        pricePerUnit
        totalPrice
        timestamp
        contract_id
        trade_route_id
        mining_waypoint_symbol
        construction_shipment_id
        isFuel
      }
    }
  }
`);

export const GET_MY_AGENT_MINI_INFO = graphql(/* GraphQL */ `
  query GetMyAgentMiniInfo {
    runInfo {
      agent {
        accountId
        symbol
        credits
        shipCount
        startingFaction
        headquarters
        createdAt
      }
    }
    budget {
      currentFunds
      reservedAmount
      spendable
    }
  }
`);

export const GET_ALL_SHIPS = graphql(/* GraphQL */ `
  query GetAllShips {
    ships {
      symbol
      registrationRole
      engineSpeed
      nav {
        status
        systemSymbol
        waypointSymbol
        flightMode
        route {
          arrival
          departureTime
          originSymbol
          originSystemSymbol
          destinationSymbol
          destinationSystemSymbol
        }
        autoPilot {
          arrival
          departureTime
          originSymbol
          originSystemSymbol
          destinationSymbol
          destinationSystemSymbol
          distance
          fuelCost
          travelTime
          route {
            connections {
              __typename
            }
          }
        }
      }
      cargo {
        units
        capacity
        inventory {
          symbol
          units
        }
      }
      fuel {
        current
        capacity
      }
      conditions {
        engine {
          condition
          integrity
        }
        frame {
          condition
          integrity
        }
        reactor {
          condition
          integrity
        }
      }
      cooldownExpiration
      status {
        assignmentId
        tempAssignmentId
        fleetId
        tempFleetId
        waitingForApi
        waitingForManager
        status {
          __typename
          ... on ChartingStatus {
            cycle
            waitingForManager
            waypointSymbol
          }
          ... on ConstructionStatus {
            cycle
            shipmentId
            shippingStatus
            waitingForManager
          }
          ... on ContractStatus {
            contractId
            runId
            cycle
            shippingStatus
            waitingForManager
          }
          ... on ManuelStatus {
            controlled
          }
          ... on MiningStatus {
            assignment {
              __typename
              ... on ExtractorAssignment {
                extractions
                state
                waypointSymbol
              }
              ... on SiphonerAssignment {
                extractions
                state
                waypointSymbol
              }
              ... on TransporterAssignment {
                cycles
                waypointSymbol
              }
              ... on SurveyorAssignment {
                surveys
                waypointSymbol
              }
              ... on IdleAssignment {
                controlled
              }
              ... on UselessAssignment {
                controlled
              }
            }
          }
          ... on ScraperStatus {
            cycle
            waitingForManager
            waypointSymbol
            scrapDate
          }
          ... on TraderStatus {
            cycle
            shipmentId
            shippingStatus
            waitingForManager
            onSleep
          }
          ... on TransferStatus {
            assignmentId
            fleetId
            systemSymbol
          }
        }
      }
    }
  }
`);

export const GET_SHIP = graphql(/* GraphQL */ `
  query GetShip($shipSymbol: String!) {
    ship(symbol: $shipSymbol) {
      symbol
      registrationRole
      engineSpeed
      nav {
        status
        systemSymbol
        waypointSymbol
        flightMode
        route {
          arrival
          departureTime
          originSymbol
          originSystemSymbol
          destinationSymbol
          destinationSystemSymbol
        }
        autoPilot {
          arrival
          departureTime
          originSymbol
          originSystemSymbol
          destinationSymbol
          destinationSystemSymbol
          distance
          fuelCost
          travelTime
          route {
            totalAntiMatterCost
            totalFuelCost
            totalCost
            totalAntiMatter
            totalFuel
            totalJumpCooldownTime
            totalTravelTime
            totalApiRequests
            totalDistance
            connections {
              ... on NavigateConnection {
                startSymbol
                endSymbol
                navMode
                distance
                travelTime
                refuel {
                  fuelNeeded
                }
              }
              ... on JumpConnection {
                startSymbol
                endSymbol
                distance
                cooldownTime
              }
              ... on WarpConnection {
                startSymbol
                endSymbol
                navMode
                distance
                travelTime
                refuel {
                  fuelNeeded
                }
              }
            }
          }
        }
      }
      cargo {
        units
        capacity
        inventory {
          symbol
          units
        }
      }
      fuel {
        current
        capacity
      }
      conditions {
        engine {
          condition
          integrity
        }
        frame {
          condition
          integrity
        }
        reactor {
          condition
          integrity
        }
      }
      cooldownExpiration
      status {
        assignmentId
        assignment {
          __typename
          id
          siphon
          warpDrive
          fleetId
          priority
          maxPurchasePrice
          creditsThreshold
          disabled
          rangeMin
          cargoMin
          survey
          extractor
        }
        tempAssignmentId
        tempAssignment {
          __typename
          id
          siphon
          warpDrive
          fleetId
          priority
          maxPurchasePrice
          creditsThreshold
          disabled
          rangeMin
          cargoMin
          survey
          extractor
        }
        fleetId
        fleet {
          id
          fleetType
          active
          systemSymbol
        }
        tempFleetId
        tempFleet {
          id
          fleetType
          active
          systemSymbol
        }
        waitingForApi
        waitingForManager
        status {
          __typename
          ... on ChartingStatus {
            cycle
            waitingForManager
            waypointSymbol
          }
          ... on ConstructionStatus {
            cycle
            shipmentId
            shippingStatus
            waitingForManager
          }
          ... on ContractStatus {
            contractId
            runId
            cycle
            shippingStatus
            waitingForManager
          }
          ... on ManuelStatus {
            controlled
          }
          ... on MiningStatus {
            assignment {
              __typename
              ... on ExtractorAssignment {
                extractions
                state
                waypointSymbol
              }
              ... on SiphonerAssignment {
                extractions
                state
                waypointSymbol
              }
              ... on TransporterAssignment {
                cycles
                waypointSymbol
              }
              ... on SurveyorAssignment {
                surveys
                waypointSymbol
              }
              ... on IdleAssignment {
                controlled
              }
              ... on UselessAssignment {
                controlled
              }
            }
          }
          ... on ScraperStatus {
            cycle
            waitingForManager
            waypointSymbol
            scrapDate
          }
          ... on TraderStatus {
            cycle
            shipmentId
            shippingStatus
            waitingForManager
            onSleep
          }
          ... on TransferStatus {
            assignmentId
            fleetId
            systemSymbol
          }
        }
      }
      reactor
      frame
      possibleScraps {
        waypointSymbol
        date
      }
      purchaseTransaction {
        id
        waypointSymbol
        shipType
        price
        timestamp
      }
      marketTransactionSummary {
        allExpenses
        allIncome
        fuelExpenses
        fuelPurchaseUnits
        fuelPurchaseTransactions
        purchaseUnits
        sellUnits
        purchaseTransactions
        sellTransactions
        allPurchaseUnits
        allPurchaseTransactions
      }
      repairTransactions {
        items {
          id
          waypointSymbol
          shipSymbol
          timestamp
          totalPrice
        }
      }
      shipModificationTransactions {
        items {
          id
          waypointSymbol
          shipSymbol
          timestamp
          tradeSymbol
          totalPrice
        }
      }
      chartTransactions {
        items {
          waypointSymbol
          shipSymbol
          totalPrice
          timestamp
          waypoint {
            symbol
            waypointType
            traits
          }
        }
      }
      constructionShipments {
        items {
          id
          tradeSymbol
          materialId
          constructionSiteWaypoint
          shipSymbol
          units
          status
          purchaseSiteWaypoint
          createdAt
          marketTransactionSummary {
            allExpenses
          }
        }
      }
      contractShipments {
        items {
          id
          contractId
          shipSymbol
          tradeSymbol
          units
          destinationSymbol
          purchaseSymbol
          createdAt
          updatedAt
          status
        }
      }
      tradeRoutes {
        items {
          id
          createdAt
          symbol
          shipSymbol
          PurchaseWaypointSymbol
          SellWaypointSymbol
          status
          tradeMode
          tradeVolume
          reservedFund
          estimatedFuel
          marketTransactionSummary {
            allExpenses
            allIncome
          }
          purchaseMarketTradeGood {
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
          }
          sellMarketTradeGood {
            tradeVolume
            supply
            activity
            purchasePrice
            sellPrice
          }
        }
      }
      surveys {
        items {
          shipInfoBefore
          updatedAt
          shipInfoAfter
          signature
          size
          waypointSymbol
          deposits
          exhaustedSince
          createdAt
          expiration
        }
      }
      extractions {
        items {
          id
          waypointSymbol
          shipSymbol
          shipInfoBefore
          shipInfoAfter
          siphon
          yieldSymbol
          yieldUnits
          survey_signature
        }
      }
      engineInfo {
        symbol
        speed
        powerRequired
        crewRequired
        slotsRequired
      }
      frameInfo {
        symbol
        moduleSlots
        mountingPoints
        fuelCapacity
        powerRequired
        crewRequired
        slotsRequired
      }
      reactorInfo {
        symbol
        powerOutput
        powerRequired
        crewRequired
        slotsRequired
      }
      mounts {
        mounts
        mountInfos {
          symbol
          strength
          deposits
          powerRequired
          crewRequired
          slotsRequired
        }
      }
      modules {
        modules
        moduleInfos {
          symbol
          capacity
          range
          powerRequired
          crewRequired
          slotsRequired
        }
      }
      scrapTransactions {
        items {
          id
          waypointSymbol
          shipSymbol
          timestamp
          totalPrice
        }
      }
      cooldown
      engine
    }
  }
`);

export const GET_ALL_SHIP_ROUTES = graphql(/* GraphQL */ `
  query GetAllShipsRoutes {
    shipRoutes {
      items {
        id
        from
        to
        shipSymbol
        navMode
        distance
        fuelCost
        travelTime
        shipStateBefore {
          id
          engineSpeed
          engineCondition
          frameCondition
          reactorCondition
        }
        shipStateAfter {
          id
          engineSpeed
          engineCondition
          frameCondition
          reactorCondition
        }
      }
    }
  }
`);

export const GET_ALL_TRADE_ROUTES = graphql(/* GraphQL */ `
  query GetAllTradeRoutes {
    tradeRoutes {
      items {
        id
        createdAt
        symbol
        shipSymbol
        PurchaseWaypointSymbol
        SellWaypointSymbol
        status
        tradeMode
        tradeVolume
        reservedFund
        estimatedFuel
        marketTransactionSummary {
          allExpenses
          allIncome
        }
        purchaseMarketTradeGood {
          tradeVolume
          supply
          activity
          purchasePrice
          sellPrice
        }
        sellMarketTradeGood {
          tradeVolume
          supply
          activity
          purchasePrice
          sellPrice
        }
      }
    }
  }
`);

export const GET_OPEN_ASSIGNMENTS = graphql(/* GraphQL */ `
  query GetOpenAssignments {
    shipAssignments(by: { open: true }) {
      items {
        id
        siphon
        warpDrive
        fleet {
          id
          fleetType
          active
          systemSymbol
        }
        fleetId
        priority
        maxPurchasePrice
        creditsThreshold
        disabled
        rangeMin
        cargoMin
        survey
        extractor
      }
    }
  }
`);

export const GET_CONTRACTS = graphql(/* GraphQL */ `
  query GetContracts {
    contracts {
      items {
        id
        factionSymbol
        contractType
        accepted
        fulfilled
        deadlineToAccept
        onAccepted
        onFulfilled
        deadline
        marketTransactionSummary {
          allExpenses
          allIncome
        }
      }
    }
    contractManager {
      runningShipments {
        id
        shipSymbol
        tradeSymbol
        units
        destinationSymbol
        purchaseSymbol
        createdAt
        updatedAt
        status
      }
    }
  }
`);

export const GET_CONTRACT = graphql(/* GraphQL */ `
  query GetContract($contractId: String!) {
    contract(symbol: $contractId) {
      id
      contractType
      accepted
      deadlineToAccept
      factionSymbol
      fulfilled
      deadline
      onAccepted
      onFulfilled
      marketTransactionSummary {
        expenses
        fuelExpenses
        allExpenses
        allIncome
      }
      reservation {
        id
        amount
        actualAmount
        status
      }
      deliveries {
        items {
          contractId
          tradeSymbol
          destinationSymbol
          unitsRequired
          unitsFulfilled
        }
      }
      shipments {
        items {
          id
          shipSymbol
          tradeSymbol
          units
          destinationSymbol
          purchaseSymbol
          createdAt
          updatedAt
          status
        }
      }
      marketTransactions {
        items {
          id
          waypointSymbol
          shipSymbol
          tradeSymbol
          type
          units
          pricePerUnit
          totalPrice
          timestamp
          contract_id
          trade_route_id
          mining_waypoint_symbol
        }
      }
    }
  }
`);

export const GET_MINING_ASSIGNMENTS = graphql(/* GraphQL */ `
  query GetMiningAssignments {
    miningManager {
      busy
      channelState {
        usedCapacity
        state
      }
      getAssignments {
        waypointSymbol
        assignedShips {
          shipSymbol
          level
        }
        lastUpdated
      }
    }
    fleets(by: { type: MINING }) {
      items {
        id
        systemSymbol
        fleetType
        active
        assignments {
          items {
            id
            priority
            rangeMin
            cargoMin
            ship {
              symbol
            }
          }
        }
        config {
          __typename
          ... on MiningConfig {
            miningEjectList
            miningTransportersPerWaypoint
            minTransporterCargoSpace
            minMiningCargoSpace
            minSiphonCargoSpace
            miningPreferList
            ignoreEngineeredAsteroids
            stopAllUnstable
            unstableSinceTimeout
            miningWaypoints
            syphonWaypoints
            minersPerWaypoint
            siphonersPerWaypoint
            surveyersPerWaypoint
          }
        }
      }
    }
  }
`);

export const GET_TOTAL_SUPPLY_CHAIN = graphql(/* GraphQL */ `
  query GetTotalSupplyChain {
    tradeSymbolInfos {
      symbol
      requires {
        items {
          symbol
          requires {
            items {
              symbol
              requires {
                items {
                  symbol
                  requires {
                    items {
                      symbol
                      requires {
                        items {
                          symbol
                          requires {
                            items {
                              symbol
                              requires {
                                items {
                                  symbol
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
      requiredBy {
        items {
          symbol
        }
      }
    }
  }
`);

export const GET_TRADE_SYMBOL_SUPPLY_CHAIN = graphql(/* GraphQL */ `
  query GetTradeSymbolSupplyChain($tradeSymbol: TradeSymbol!) {
    tradeSymbolInfo(tradeSymbol: $tradeSymbol) {
      symbol
      requires {
        items {
          symbol
          requires {
            items {
              symbol
              requires {
                items {
                  symbol
                  requires {
                    items {
                      symbol
                      requires {
                        items {
                          symbol
                          requires {
                            items {
                              symbol
                              requires {
                                items {
                                  symbol
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
      requiredBy {
        items {
          symbol
        }
      }
    }
    marketTrades(by: { tradeSymbol: $tradeSymbol }) {
      items {
        symbol
        waypointSymbol
        type
        createdAt
        marketTradeGood {
          symbol
          waypointSymbol
          type
          tradeVolume
          supply
          activity
          purchasePrice
          sellPrice
          createdAt
        }
      }
    }
  }
`);

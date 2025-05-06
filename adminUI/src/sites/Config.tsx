import {
  CodeOutlined,
  RocketOutlined,
  SaveOutlined,
  SettingOutlined,
  ShoppingOutlined,
} from "@ant-design/icons";
import {
  Button,
  Col,
  Flex,
  Input,
  InputNumber,
  Row,
  Select,
  Switch,
  Tabs,
  TabsProps,
  Typography,
} from "antd";
import { useEffect, useState } from "react";
import { backendUrl } from "../data";
import PageTitle from "../features/PageTitle";
import { TradeSymbol } from "../models/api";

const { Text } = Typography;

export function ConfigScreen() {
  const [config, setConfig] = useState({
    antimatter_price: 0,
    control_active: false,
    control_start_sleep: 0,
    default_profit: 0,
    default_purchase_price: 0,
    default_sell_price: 0,
    extra_mining_transporter: 0,
    fuel_cost: 0,
    ignore_engineered_asteroids: false,
    margin_percentage: 0,
    market_blacklist: [],
    markets_per_ship: 0,
    markup_percentage: 0,
    max_miners_per_waypoint: 0,
    mining_eject_list: [],
    mining_prefer_list: [],
    mining_ships_per_waypoint: 0,
    mining_waypoints_per_system: 0,
    purchase_multiplier: 0,
    scrap_agents: false,
    scrapper_start_sleep: 0,
    socket_address: "",
    stop_all_unstable: false,
    trade_mode: "ProfitPerHour",
    transport_capacity_per_waypoint: 0,
    unstable_since_timeout: 0,
    update_all_systems: false,
  });

  useEffect(() => {
    fetch(`http://${backendUrl}/insights/config`)
      .then((response) => response.json())
      .then((response) => {
        console.log(response);
        setConfig(response);
      });
  }, []);

  const handleChange = (
    field: string,
    value: string | number | boolean | string[] | null
  ) => {
    setConfig({ ...config, [field]: value });
  };

  const tradeModes = [
    { label: "Profit Per Hour", value: "ProfitPerHour" },
    { label: "Profit Per API Request", value: "ProfitPerAPIRequest" },
    { label: "Profit Per Trip", value: "ProfitPerTrip" },
  ];

  const saveConfig = () => {
    console.log("Configuration saved:", config);
    // In a real application, this would send the config to your backend
    fetch(`http://${backendUrl}/insights/config`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(config),
    })
      .then((response) => response.json())
      .then((response) => {
        console.log(response);
        setConfig(response);
      });
  };

  const FormItem = ({
    label,
    children,
  }: {
    label: React.ReactNode;
    children: React.ReactNode;
  }) => (
    <div style={{ marginBottom: "16px" }}>
      <div style={{ marginBottom: "8px" }}>
        <Text strong>{label}</Text>
      </div>
      {children}
    </div>
  );

  const tabItems: TabsProps["items"] = [
    {
      key: "system",
      label: "System Settings",
      icon: <SettingOutlined />,
      children: (
        <div>
          <Row gutter={16}>
            <Col span={12}>
              <FormItem label="Control Active">
                <Switch
                  checked={config.control_active}
                  onChange={(checked) =>
                    handleChange("control_active", checked)
                  }
                />
              </FormItem>
            </Col>
            <Col span={12}>
              <FormItem label="Control Start Sleep">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.control_start_sleep}
                  onChange={(value) =>
                    handleChange("control_start_sleep", value)
                  }
                />
              </FormItem>
            </Col>
          </Row>

          <Row gutter={16}>
            <Col span={6}>
              <FormItem label="Update All Systems">
                <Switch
                  checked={config.update_all_systems}
                  onChange={(checked) =>
                    handleChange("update_all_systems", checked)
                  }
                />
              </FormItem>
            </Col>
            <Col span={6}>
              <FormItem label="Scrap Agents">
                <Switch
                  checked={config.scrap_agents}
                  onChange={(checked) => handleChange("scrap_agents", checked)}
                />
              </FormItem>
            </Col>
            <Col span={12}>
              <FormItem label="Scrapper Start Sleep">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.scrapper_start_sleep}
                  onChange={(value) =>
                    handleChange("scrapper_start_sleep", value)
                  }
                />
              </FormItem>
            </Col>
          </Row>

          <FormItem label="Socket Address">
            <Input
              value={config.socket_address}
              onChange={(e) => handleChange("socket_address", e.target.value)}
            />
          </FormItem>
        </div>
      ),
    },
    {
      key: "transport",
      label: "Transport & Market",
      icon: <ShoppingOutlined />,
      children: (
        <div>
          <Row gutter={16}>
            <Col span={12}>
              <FormItem label="Antimatter Price">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.antimatter_price}
                  onChange={(value) => handleChange("antimatter_price", value)}
                />
              </FormItem>
            </Col>
            <Col span={12}>
              <FormItem label="Fuel Cost">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.fuel_cost}
                  onChange={(value) => handleChange("fuel_cost", value)}
                />
              </FormItem>
            </Col>
          </Row>

          <Row gutter={16}>
            <Col span={8}>
              <FormItem label="Default Profit">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.default_profit}
                  onChange={(value) => handleChange("default_profit", value)}
                />
              </FormItem>
            </Col>
            <Col span={8}>
              <FormItem label="Default Purchase Price">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.default_purchase_price}
                  onChange={(value) =>
                    handleChange("default_purchase_price", value)
                  }
                />
              </FormItem>
            </Col>
            <Col span={8}>
              <FormItem label="Default Sell Price">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.default_sell_price}
                  onChange={(value) =>
                    handleChange("default_sell_price", value)
                  }
                />
              </FormItem>
            </Col>
          </Row>

          <Row gutter={16}>
            <Col span={8}>
              <FormItem label="Margin Percentage">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.margin_percentage}
                  onChange={(value) => handleChange("margin_percentage", value)}
                  step={0.01}
                  precision={2}
                />
              </FormItem>
            </Col>
            <Col span={8}>
              <FormItem label="Markup Percentage">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.markup_percentage}
                  onChange={(value) => handleChange("markup_percentage", value)}
                  step={0.01}
                  precision={2}
                />
              </FormItem>
            </Col>
            <Col span={8}>
              <FormItem label="Purchase Multiplier">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.purchase_multiplier}
                  onChange={(value) =>
                    handleChange("purchase_multiplier", value)
                  }
                  step={0.1}
                  precision={1}
                />
              </FormItem>
            </Col>
          </Row>
          <Row gutter={16}>
            <Col span={12}>
              <FormItem label="Trade Mode">
                <Select
                  style={{ width: "100%" }}
                  value={config.trade_mode}
                  onChange={(value) => handleChange("trade_mode", value)}
                  options={tradeModes}
                />
              </FormItem>
            </Col>
            <Col span={12}>
              <FormItem label="Markets Per Ship">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.markets_per_ship}
                  onChange={(value) => handleChange("markets_per_ship", value)}
                />
              </FormItem>
            </Col>
          </Row>

          <FormItem label="Market Blacklist">
            <Select
              mode="tags"
              style={{ width: "100%" }}
              placeholder="Add items to blacklist"
              value={config.market_blacklist}
              options={Object.values(TradeSymbol).map((symbol) => ({
                label: symbol,
                value: symbol,
              }))}
              onChange={(values) => handleChange("market_blacklist", values)}
            />
          </FormItem>
        </div>
      ),
    },

    {
      key: "mining",
      label: "Mining Settings",
      icon: <RocketOutlined />,
      children: (
        <div>
          <Row gutter={16}>
            <Col span={8}>
              <FormItem label="Mining Ships Per Waypoint">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.mining_ships_per_waypoint}
                  onChange={(value) =>
                    handleChange("mining_ships_per_waypoint", value)
                  }
                />
              </FormItem>
            </Col>
            <Col span={8}>
              <FormItem label="Max Miners Per Waypoint">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.max_miners_per_waypoint}
                  onChange={(value) =>
                    handleChange("max_miners_per_waypoint", value)
                  }
                />
              </FormItem>
            </Col>
            <Col span={8}>
              <FormItem label="Mining Waypoints Per System">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.mining_waypoints_per_system}
                  onChange={(value) =>
                    handleChange("mining_waypoints_per_system", value)
                  }
                />
              </FormItem>
            </Col>
          </Row>

          <Row gutter={16}>
            <Col span={12}>
              <FormItem label="Transport Capacity Per Waypoint">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.transport_capacity_per_waypoint}
                  onChange={(value) =>
                    handleChange("transport_capacity_per_waypoint", value)
                  }
                />
              </FormItem>
            </Col>
            <Col span={12}>
              <FormItem label="Extra Mining Transporter">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.extra_mining_transporter}
                  onChange={(value) =>
                    handleChange("extra_mining_transporter", value)
                  }
                />
              </FormItem>
            </Col>
          </Row>

          <Row>
            <Col span={6}>
              <FormItem label="Ignore Engineered Asteroids">
                <Switch
                  checked={config.ignore_engineered_asteroids}
                  onChange={(checked) =>
                    handleChange("ignore_engineered_asteroids", checked)
                  }
                />
              </FormItem>
            </Col>
            <Col span={6}>
              <FormItem label="Stop All Unstable">
                <Switch
                  checked={config.stop_all_unstable}
                  onChange={(checked) =>
                    handleChange("stop_all_unstable", checked)
                  }
                />
              </FormItem>
            </Col>
            <Col span={12}>
              <FormItem label="Unstable Since Timeout">
                <InputNumber
                  style={{ width: "100%" }}
                  value={config.unstable_since_timeout}
                  onChange={(value) =>
                    handleChange("unstable_since_timeout", value)
                  }
                />
              </FormItem>
            </Col>
          </Row>

          <FormItem label="Mining Prefer List">
            <Select
              mode="tags"
              style={{ width: "100%" }}
              placeholder="Add mining preferences"
              value={config.mining_prefer_list}
              options={Object.values(TradeSymbol).map((symbol) => ({
                label: symbol,
                value: symbol,
              }))}
              onChange={(values) => handleChange("mining_prefer_list", values)}
            />
          </FormItem>

          <FormItem label="Mining Eject List">
            <Select
              mode="tags"
              style={{ width: "100%" }}
              placeholder="Add items to eject"
              value={config.mining_eject_list}
              options={Object.values(TradeSymbol).map((symbol) => ({
                label: symbol,
                value: symbol,
              }))}
              onChange={(values) => handleChange("mining_eject_list", values)}
            />
          </FormItem>
        </div>
      ),
    },
    {
      key: "json",
      label: (
        <span>
          <CodeOutlined />
          JSON Config
        </span>
      ),
      children: (
        <div>
          <pre>{JSON.stringify(config, null, 2)}</pre>
        </div>
      ),
    },
  ];

  return (
    <div style={{ padding: "24px" }}>
      <PageTitle title="Configuration" />
      <Flex
        justify="space-between"
        align="center"
        style={{ marginBottom: "24px" }}
      >
        <h1>Configuration</h1>
        <Button icon={<SaveOutlined />} onClick={saveConfig}>
          Save
        </Button>
      </Flex>

      <Tabs
        defaultActiveKey="economic"
        items={tabItems}
        type="card"
        style={{ marginBottom: "24px" }}
      />
    </div>
  );
}

import { EditOutlined } from "@ant-design/icons";
import { useMutation } from "@apollo/client/react";
import {
  Alert,
  Button,
  Descriptions,
  DescriptionsProps,
  Form,
  Input,
  InputNumber,
  Modal,
  Select,
  Space,
  Switch,
} from "antd";
import { useState, type ReactNode } from "react";
import {
  ConstructionMode,
  FleetConfig,
  InputFleetConfig,
  TradeMode,
  TradeSymbol,
} from "../../gql/graphql";
import { EDIT_FLEET } from "../../graphql/mutations";
import { GET_FLEET } from "../../graphql/queries";

type ConfigFieldType =
  | "boolean"
  | "int"
  | "float"
  | "string"
  | "enum"
  | "tradeSymbols";

type ConfigField = {
  name: string;
  label: string;
  type: ConfigFieldType;
  options?: string[];
};

type ConfigEditor = {
  key: string;
  fields: ConfigField[];
};

const CONSTRUCTION_MODE_OPTIONS = Object.values(ConstructionMode);
const TRADE_MODE_OPTIONS = Object.values(TradeMode);
const TRADE_SYMBOL_OPTIONS = Object.values(TradeSymbol);

function toOption(value: string) {
  return { value, label: value };
}

function getConfigEditor(config: FleetConfig): ConfigEditor {
  switch (config.__typename) {
    case "ChartingConfig":
      return {
        key: "charting",
        fields: [
          {
            name: "chartOnlyJumpGates",
            label: "Chart Only Jump Gates",
            type: "boolean",
          },
          {
            name: "chartingProbeCount",
            label: "Charting Probe Count",
            type: "int",
          },
        ],
      };
    case "ConstructionConfig":
      return {
        key: "construction",
        fields: [
          {
            name: "constructionMode",
            label: "Construction Mode",
            type: "enum",
            options: CONSTRUCTION_MODE_OPTIONS,
          },
          {
            name: "constructionShipCount",
            label: "Construction Ship Count",
            type: "int",
          },
          {
            name: "constructionWaypoint",
            label: "Construction Waypoint",
            type: "string",
          },
        ],
      };
    case "ContractConfig":
      return {
        key: "contract",
        fields: [
          {
            name: "contractShipCount",
            label: "Contract Ship Count",
            type: "int",
          },
        ],
      };
    case "ManuelConfig":
      return {
        key: "manuel",
        fields: [{ name: "config", label: "Config", type: "string" }],
      };
    case "MiningConfig":
      return {
        key: "mining",
        fields: [
          { name: "miningWaypoints", label: "Mining Waypoints", type: "int" },
          {
            name: "ignoreEngineeredAsteroids",
            label: "Ignore Engineered Asteroids",
            type: "boolean",
          },
          {
            name: "stopAllUnstable",
            label: "Stop All Unstable",
            type: "boolean",
          },
          {
            name: "unstableSinceTimeout",
            label: "Unstable Since Timeout",
            type: "int",
          },
          { name: "syphonWaypoints", label: "Syphon Waypoints", type: "int" },
          {
            name: "minTransporterCargoSpace",
            label: "Minimum Transporter Cargo Space",
            type: "int",
          },
          {
            name: "minMiningCargoSpace",
            label: "Minimum Mining Cargo Space",
            type: "int",
          },
          {
            name: "minSiphonCargoSpace",
            label: "Minimum Siphon Cargo Space",
            type: "int",
          },
          {
            name: "surveyersPerWaypoint",
            label: "Surveyers Per Waypoint",
            type: "int",
          },
          {
            name: "minersPerWaypoint",
            label: "Miners Per Waypoint",
            type: "int",
          },
          {
            name: "miningTransportersPerWaypoint",
            label: "Transporters Per Waypoint",
            type: "int",
          },
          {
            name: "siphonersPerWaypoint",
            label: "Siphoners Per Waypoint",
            type: "int",
          },
          {
            name: "miningEjectList",
            label: "Mining Eject List",
            type: "tradeSymbols",
          },
          {
            name: "miningPreferList",
            label: "Mining Prefer List",
            type: "tradeSymbols",
          },
        ],
      };
    case "ScrapingConfig":
      return {
        key: "scraping",
        fields: [
          {
            name: "shipMarketRatio",
            label: "Ship Market Ratio",
            type: "float",
          },
          { name: "allowedRequests", label: "Allowed Requests", type: "int" },
          {
            name: "notifyOnShipyard",
            label: "Notify on Shipyard",
            type: "boolean",
          },
        ],
      };
    case "TradingConfig":
      return {
        key: "trading",
        fields: [
          {
            name: "tradeMode",
            label: "Trade Mode",
            type: "enum",
            options: TRADE_MODE_OPTIONS,
          },
          {
            name: "tradeProfitThreshold",
            label: "Trade Profit Threshold",
            type: "int",
          },
          { name: "minCargoSpace", label: "Minimum Cargo Space", type: "int" },
          {
            name: "shipMarketRatio",
            label: "Ship Market Ratio",
            type: "float",
          },
          {
            name: "purchaseMultiplier",
            label: "Purchase Multiplier",
            type: "float",
          },
          {
            name: "marketBlacklist",
            label: "Market Blacklist",
            type: "tradeSymbols",
          },
          {
            name: "marketPreferList",
            label: "Market Prefer List",
            type: "tradeSymbols",
          },
        ],
      };
    default:
      return { key: "", fields: [] };
  }
}

function configInitialValues(config: FleetConfig, fields: ConfigField[]) {
  const values: Record<string, unknown> = {};
  for (const field of fields) {
    values[field.name] = (config as unknown as Record<string, unknown>)[
      field.name
    ];
  }
  return values;
}

function buildConfigInput(
  config: FleetConfig,
  values: Record<string, unknown>,
): InputFleetConfig {
  const { key, fields } = getConfigEditor(config);
  const fieldValues: Record<string, unknown> = {};
  for (const field of fields) {
    fieldValues[field.name] = values[field.name];
  }
  return { [key]: fieldValues } as unknown as InputFleetConfig;
}

function renderConfigValue(config: FleetConfig, field: ConfigField): ReactNode {
  const value = (config as unknown as Record<string, unknown>)[field.name];
  if (field.type === "boolean") {
    return value === true ? "Yes" : "No";
  }
  if (field.type === "tradeSymbols") {
    const list = value as string[];
    return (
      <div>
        {list.toSorted().map((entry) => (
          <div key={entry}>{entry}</div>
        ))}
      </div>
    );
  }
  return String(value);
}

function renderInput(field: ConfigField) {
  switch (field.type) {
    case "boolean":
      return <Switch />;
    case "int":
      return <InputNumber precision={0} style={{ width: "100%" }} />;
    case "float":
      return <InputNumber style={{ width: "100%" }} />;
    case "string":
      return <Input />;
    case "enum":
      return <Select options={(field.options ?? []).map(toOption)} />;
    case "tradeSymbols":
      return (
        <Select
          mode="multiple"
          options={TRADE_SYMBOL_OPTIONS.map(toOption)}
          allowClear
        />
      );
    default:
      return <Input />;
  }
}

function FleetConfigDetails({
  config,
  fleetId,
}: {
  config: FleetConfig;
  fleetId: number;
}) {
  const [open, setOpen] = useState(false);
  const [form] = Form.useForm();

  const [editFleet, { loading: editLoading, error: editError }] = useMutation(
    EDIT_FLEET,
    {
      refetchQueries: [{ query: GET_FLEET, variables: { fleetID: fleetId } }],
    },
  );

  const editor = getConfigEditor(config);
  const items: NonNullable<DescriptionsProps["items"]> = [
    { key: "type", label: "Type", children: config.__typename },
    ...editor.fields.map((field) => ({
      key: field.name,
      label: field.label,
      children: renderConfigValue(config, field),
    })),
  ];

  if (editor.fields.length === 0) {
    items.push({ key: "config", label: "Config", children: "No Config Found" });
  }

  const handleOk = async () => {
    try {
      const values = (await form.validateFields()) as Record<string, unknown>;
      await editFleet({
        variables: { id: fleetId, config: buildConfigInput(config, values) },
      });
      setOpen(false);
    } catch {
      // Keep the modal open on validation or mutation errors.
    }
  };

  return (
    <>
      <Descriptions
        bordered
        size="small"
        column={4}
        items={items}
        title={
          <Space>
            <span>Config ({config.__typename})</span>
            <Button
              type="text"
              size="small"
              icon={<EditOutlined />}
              onClick={() => setOpen(true)}
              aria-label={`Edit ${config.__typename}`}
            />
          </Space>
        }
      />
      <Modal
        open={open}
        title={`Edit ${config.__typename}`}
        okText="Apply"
        onOk={handleOk}
        onCancel={() => setOpen(false)}
        confirmLoading={editLoading}
        destroyOnClose
      >
        {editError && (
          <Alert
            message="Failed to update fleet config"
            description={editError.message}
            type="error"
            showIcon
            style={{ marginBottom: 16 }}
          />
        )}
        <Form
          form={form}
          layout="vertical"
          initialValues={configInitialValues(config, editor.fields)}
        >
          {editor.fields.map((field) => (
            <Form.Item
              key={field.name}
              name={field.name}
              label={field.label}
              valuePropName={field.type === "boolean" ? "checked" : undefined}
            >
              {renderInput(field)}
            </Form.Item>
          ))}
        </Form>
      </Modal>
    </>
  );
}

export default FleetConfigDetails;

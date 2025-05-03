import {
  AuditOutlined,
  ContactsOutlined,
  DeliveredProcedureOutlined,
  FileTextOutlined,
  GlobalOutlined,
  HomeOutlined,
  InfoCircleOutlined,
  ReconciliationOutlined,
  RocketOutlined,
  ShoppingCartOutlined,
  TeamOutlined,
  TruckOutlined,
} from "@ant-design/icons";
import type { MenuProps } from "antd";
import { Menu } from "antd";
import { Link } from "react-router-dom";
import type { AntSiderSider } from "../MyApp";
import { useAppDispatch, useAppSelector } from "../redux/hooks";
import {
  selectSliderCollapsed,
  setSliderCollapsed,
} from "../redux/slices/configSlice";
import FaIcon from "./FontAwsome/FaIcon";

type MenuItem = Required<MenuProps>["items"][number];

const items: MenuItem[] = [
  // Main navigation
  {
    label: <Link to="/">Spacetraders API</Link>,
    key: "home",
    icon: <HomeOutlined />,
  },

  // Fleet management
  {
    key: "fleet",
    label: "Fleet Management",
    icon: <RocketOutlined />,
    children: [
      {
        label: <Link to="/ships">Ships</Link>,
        key: "ships",
        icon: <RocketOutlined />,
      },
      {
        label: <Link to="/shipsToPurchase">Ships To Purchase</Link>,
        key: "shipsToPurchase",
        icon: <FaIcon type="solid" icon="fa-cart-plus" />,
      },
      {
        label: <Link to="/fleet/selected">Selected Ship</Link>,
        key: "fleet/selected",
        icon: <RocketOutlined />,
      },
      {
        label: <Link to="/miningAssignments">Mining Assignments</Link>,
        key: "miningAssignments",
        icon: <FaIcon type="solid" icon="fa-excavator" />,
      },
      {
        label: <Link to="/bulk">Bulk Actions</Link>,
        key: "bulk",
        icon: <ReconciliationOutlined />,
      },
    ],
  },

  // Trade & Economy
  {
    key: "trade",
    label: "Trade & Economy",
    icon: <ShoppingCartOutlined />,
    children: [
      {
        label: <Link to="/contracts">Contracts</Link>,
        key: "contracts",
        icon: <FileTextOutlined />,
      },
      {
        label: <Link to="/tradeRoutes">Trade Routes</Link>,
        key: "tradeRoutes",
        icon: <ShoppingCartOutlined />,
      },
      {
        label: <Link to="/possibleTrades">Possible Trades</Link>,
        key: "possibleTrades",
        icon: <AuditOutlined />,
      },
      {
        label: <Link to="/transactions/market">Market Transactions</Link>,
        key: "transactions/market",
        icon: <DeliveredProcedureOutlined />,
      },
    ],
  },

  // Construction
  {
    key: "construction",
    label: "Construction",
    icon: <FaIcon type="regular" icon="fa-person-digging" />,
    children: [
      {
        label: <Link to="/construction/Materials">Constructions</Link>,
        key: "construction/Materials",
        icon: <FaIcon type="regular" icon="fa-person-digging" />,
      },
      {
        label: <Link to="/construction/shipments">Construction Shipments</Link>,
        key: "construction/shipments",
        icon: <TruckOutlined />,
      },
    ],
  },

  // Universe & Navigation
  {
    key: "universe",
    label: "Universe & Navigation",
    icon: <GlobalOutlined />,
    children: [
      {
        label: <Link to="/systems">Systems</Link>,
        key: "systems",
        icon: <GlobalOutlined />,
      },
      {
        label: <Link to="/systems/map">Systems Map</Link>,
        key: "systems/maps",
        icon: <FaIcon type="solid" icon="fa-map-location-dot" />,
      },
      {
        label: <Link to="/system/selected">Selected System</Link>,
        key: "system/selected",
        icon: <GlobalOutlined />,
      },
      {
        label: <Link to="/system/selected/selected">Selected Waypoint</Link>,
        key: "system/selected/selected",
        icon: <GlobalOutlined />,
      },
      {
        label: <Link to="/system/wpConfig">Wp Map Config</Link>,
        key: "map",
        icon: <FaIcon type="solid" icon="fa-location-dot" />,
      },
    ],
  },

  // Game Info
  {
    key: "gameInfo",
    label: "Game Info",
    icon: <InfoCircleOutlined />,
    children: [
      {
        label: <Link to="/agents">Agents</Link>,
        key: "agents",
        icon: <ContactsOutlined />,
      },
      {
        label: <Link to="/factions">Factions</Link>,
        key: "factions",
        icon: <TeamOutlined />,
      },
      {
        label: <Link to="/surveys">Surveys</Link>,
        key: "surveys",
        icon: <AuditOutlined />,
      },
    ],
  },
];

function MySider({ Slider }: { Slider: typeof AntSiderSider }) {
  // const [collapsed, setCollapsed] = useState(false);

  const collapsed = useAppSelector(selectSliderCollapsed);
  const dispatch = useAppDispatch();

  return (
    <Slider
      collapsible
      collapsed={collapsed}
      onCollapse={(value) => dispatch(setSliderCollapsed(value))}
      theme="light"
      width={240}
    >
      <Menu mode="inline" items={items}></Menu>
    </Slider>
  );
}

export default MySider;

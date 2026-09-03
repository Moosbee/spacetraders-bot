import {
  NodeIndexOutlined,
  RocketOutlined,
  SortDescendingOutlined,
  TruckOutlined,
} from "@ant-design/icons";
import { Flex } from "antd";
import { GetFleetsQuery } from "../../gql/graphql";

type Assignment =
  GetFleetsQuery["fleets"]["items"][number]["assignments"]["items"][number];

function AssignmentsPopover({ assignments }: { assignments: Assignment[] }) {
  return (
    <Flex flex={1} vertical>
      {assignments.map((asgmt) => (
        <Flex key={asgmt.id} justify="space-between">
          {asgmt.ship.length} |{asgmt.id} {asgmt.disabled ? "D" : "A"}|
          <SortDescendingOutlined /> {asgmt.priority}|
          <NodeIndexOutlined /> {asgmt.rangeMin}|
          <TruckOutlined /> {asgmt.cargoMin}|
          <RocketOutlined /> {(asgmt.ship ?? []).length}|
          {asgmt.extractor && "E|"}
          {asgmt.siphon && "SI|"}
          {asgmt.survey && "SU|"}
          {asgmt.warpDrive && "W|"}
        </Flex>
      ))}
    </Flex>
  );
}

export default AssignmentsPopover;

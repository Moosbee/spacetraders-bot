import { useMutation } from "@apollo/client/react";
import { Alert, Button, Checkbox, Flex, InputNumber, Space } from "antd";
import { useState } from "react";
import { FORCE_ASSIGN_SHIP } from "../../graphql/mutations";
import { GET_SHIP } from "../../graphql/queries";
import { ShipData } from "../../sites/Ship";

function ShipControls({ ship }: { ship: ShipData }) {
  void ship;
  return (
    <Flex justify="center" align="flex-start" vertical>
      <ForceAssignShipControls ship={ship} />
    </Flex>
  );
}

function ForceAssignShipControls({ ship }: { ship: ShipData }) {
  const [temp, setTemp] = useState(false);
  const [assignmentId, setAssignmentId] = useState<number | null>(
    ship.status.currentAssignmentId ?? null,
  );
  const [
    forceAssignShip,
    { loading: forceAssignLoading, error: forceAssignError },
  ] = useMutation(FORCE_ASSIGN_SHIP, {
    refetchQueries: [
      { query: GET_SHIP, variables: { shipSymbol: ship.symbol } },
    ],
  });
  return (
    <Space>
      <InputNumber
        placeholder="ID"
        min={0}
        value={assignmentId}
        onChange={(value) => setAssignmentId(value)}
      />
      <Button
        disabled={assignmentId === null || forceAssignLoading}
        loading={forceAssignLoading}
        onClick={() => {
          if (assignmentId === null) {
            return;
          }
          forceAssignShip({
            variables: {
              shipSymbol: ship.symbol,
              assignmentId,
              temp,
            },
          });
        }}
      >
        Force Assign
      </Button>
      <Checkbox checked={temp} onChange={() => setTemp(!temp)}>
        Temp
      </Checkbox>
      {forceAssignError && (
        <Alert message={forceAssignError.message} type="error" />
      )}
    </Space>
  );
}

export default ShipControls;

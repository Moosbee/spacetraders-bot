import { useQuery } from "@apollo/client/react";
import { Button, Descriptions, Divider, Result, Space, Spin } from "antd";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { GET_SHIP_TRANSFER } from "../graphql/queries";

function ShipTransfer() {
  const { shipTransferID } = useParams();

  const { loading, error, data, refetch } = useQuery(GET_SHIP_TRANSFER, {
    variables: { shipTransferId: Number(shipTransferID) },
  });

  const transfer = data?.shipTransfer;

  if (error) {
    return (
      <div style={{ padding: "24px 24px" }}>
        <PageTitle title={`Ship Transfer ${shipTransferID}`} />
        <Result
          status="error"
          title="Ship Transfer Error"
          subTitle={error.message}
          extra={[
            <Button key="retry" type="primary" onClick={() => refetch()}>
              Try Again
            </Button>,
          ]}
        />
      </div>
    );
  }

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Ship Transfer ${shipTransferID}`} />
      <Spin spinning={loading}>
        <Space wrap>
          <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
            Ship Transfer {shipTransferID}
          </h1>
          <Button onClick={() => refetch()} loading={loading}>
            Reload
          </Button>
        </Space>
        <Divider />
        {transfer ? (
          <Descriptions
            bordered
            size="small"
            column={2}
            items={[
              {
                key: "id",
                label: "ID",
                children: transfer.id,
              },
              {
                key: "state",
                label: "State",
                children: transfer.state,
              },
              {
                key: "ship",
                label: "Ship",
                children: (
                  <Link to={`/ships/${transfer.shipSymbol}`}>
                    {transfer.ship?.symbol ?? transfer.shipSymbol}
                  </Link>
                ),
              },
              {
                key: "fleet",
                label: "Fleet",
                children: (
                  <Link to={`/fleets/${transfer.fleetId}`}>
                    {transfer.fleet
                      ? `${transfer.fleet.fleetType} ${transfer.fleetId}`
                      : transfer.fleetId}
                  </Link>
                ),
              },
              {
                key: "assignment",
                label: "Assignment",
                children: transfer.assignment
                  ? `${transfer.assignmentId} (fleet ${transfer.assignment.fleetId})`
                  : transfer.assignmentId,
              },
              {
                key: "reservation",
                label: "Reservation",
                children: (
                  <Space size={8}>
                    <Link to="/reservedFunds">{transfer.reservationId}</Link>
                    {transfer.reservation && (
                      <span>
                        <MoneyDisplay amount={transfer.reservation.amount} /> ·{" "}
                        {transfer.reservation.status}
                      </span>
                    )}
                  </Space>
                ),
              },
              {
                key: "startSystem",
                label: "Start System",
                children: (
                  <Link to={`/system/${transfer.startSystemSymbol}`}>
                    {transfer.startSystem?.symbol ?? transfer.startSystemSymbol}
                  </Link>
                ),
              },
              {
                key: "endSystem",
                label: "End System",
                children: (
                  <Link to={`/system/${transfer.endSystemSymbol}`}>
                    {transfer.endSystem?.symbol ?? transfer.endSystemSymbol}
                  </Link>
                ),
              },
              {
                key: "createdAt",
                label: "Created At",
                children: new Date(transfer.createdAt).toLocaleString(),
              },
              {
                key: "updatedAt",
                label: "Updated At",
                children: new Date(transfer.updatedAt).toLocaleString(),
              },
            ]}
          />
        ) : (
          !loading && (
            <Result
              status="404"
              title="Ship transfer not found"
              subTitle={`No ship transfer with id ${shipTransferID}`}
            />
          )
        )}
      </Spin>
    </div>
  );
}

export default ShipTransfer;

import { Card, List, Space } from "antd";
import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import {
  Assignment,
  MiningAssignmentsResponse,
} from "../models/MiningAssignments";
import { backendUrl } from "../store";

export default function MiningAssignments() {
  const [shipAssignments, setShipAssignments] = useState<Assignment[] | null>();

  useEffect(() => {
    fetch(`http://${backendUrl}/insights/mining/assignments`)
      .then((response) => response.json())
      .then((data: MiningAssignmentsResponse) => {
        console.log("/insights/mining/assignments", data);
        setShipAssignments(data.assignments);
      });
  }, []);
  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Mining Assignments" />
      <Space>
        <h1>
          Mining Assignments{" "}
          {shipAssignments
            ?.map((a) => Object.keys(a[1].assigned_ships).length)
            .reduce((a, b) => a + b, 0)}
        </h1>
      </Space>
      <List
        grid={{ gutter: 16, column: 4 }}
        dataSource={shipAssignments || []}
        renderItem={(item) => (
          <List.Item>
            <Card title={item[0]}>
              <List
                dataSource={Object.entries(item[1].assigned_ships)}
                renderItem={(ship) => (
                  <List.Item>
                    <Link to={`/ships/${ship[0]}`}>{ship[0]}</Link>: {ship[1]}
                  </List.Item>
                )}
              />
            </Card>
          </List.Item>
        )}
      />
    </div>
  );
}

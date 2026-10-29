import { useMutation } from "@apollo/client/react";
import { Alert, App, Form, Modal, Switch } from "antd";
import { SHUTDOWN } from "../graphql/mutations";

type ShutdownValues = {
  global: boolean;
  run: boolean;
  slowManagers: boolean;
  fastManagers: boolean;
  slowShips: boolean;
  fastShips: boolean;
};

const DEFAULT_VALUES: ShutdownValues = {
  global: false,
  run: true,
  slowManagers: false,
  fastManagers: false,
  slowShips: false,
  fastShips: false,
};

const TOKEN_LABELS: Array<[keyof ShutdownValues, string]> = [
  ["global", "Global"],
  ["run", "Run"],
  ["slowManagers", "Slow Managers"],
  ["fastManagers", "Fast Managers"],
  ["slowShips", "Slow Ships"],
  ["fastShips", "Fast Ships"],
];

function ShutdownModal({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const { modal } = App.useApp();
  const [form] = Form.useForm<ShutdownValues>();

  const [shutdown, { loading, error }] = useMutation(SHUTDOWN);

  const runShutdown = async (values: ShutdownValues) => {
    try {
      await shutdown({ variables: values });
      onClose();
    } catch {
      // Keep the modal open so the error is visible below the form.
    }
  };

  const handleShutdown = async () => {
    let values: ShutdownValues;
    try {
      values = await form.validateFields();
    } catch {
      return;
    }

    const enabledTokens = TOKEN_LABELS.filter(([key]) => values[key]).map(
      ([, label]) => label,
    );

    modal.confirm({
      title: "Shutdown the server?",
      content:
        enabledTokens.length > 0 ? (
          <div>
            <p>The following cancel tokens will be triggered:</p>
            <ul style={{ marginBottom: 0 }}>
              {enabledTokens.map((label) => (
                <li key={label}>{label}</li>
              ))}
            </ul>
            {enabledTokens.some((label) => label === "Global") && (
              <p style={{ marginTop: 8 }}>
                <strong>
                  The Global cancel token will shut down the entire server. It
                  will create a backup of the database and then shut down the
                  server. No database deletion or anything.
                </strong>
              </p>
            )}
            {enabledTokens.some((label) => label === "Run") && (
              <p style={{ marginTop: 8, color: "red" }}>
                <strong>
                  The Run cancel token will emulate a universe reset, the server
                  will create a DB-Backup and reset the database. Afterward it
                  will try to create a new agent and probably get a 409 conflict
                  error
                </strong>
              </p>
            )}
          </div>
        ) : (
          <p>
            No cancel tokens are selected. The server will not be shut down.
          </p>
        ),
      okText: "Shutdown",
      okButtonProps: { danger: true },
      onOk: () => runShutdown(values),
    });
  };

  return (
    <Modal
      open={open}
      title="Shutdown"
      okText="Shutdown"
      okButtonProps={{ danger: true }}
      confirmLoading={loading}
      onOk={handleShutdown}
      onCancel={onClose}
      destroyOnHidden
    >
      {error && (
        <Alert
          message="Shutdown failed"
          description={error.message}
          type="error"
          showIcon
          style={{ marginBottom: 16 }}
        />
      )}
      <Form
        form={form}
        layout="horizontal"
        initialValues={DEFAULT_VALUES}
        style={{ marginTop: 16 }}
        labelCol={{ span: 12 }}
        wrapperCol={{ span: 12 }}
      >
        {TOKEN_LABELS.map(([key, label]) => (
          <Form.Item
            key={key}
            name={key}
            label={`${label} Cancel Token`}
            valuePropName="checked"
            style={{
              marginBottom: 8,
            }}
          >
            <Switch />
          </Form.Item>
        ))}
      </Form>
    </Modal>
  );
}

export default ShutdownModal;

import { Button, Result } from "antd";
import PageTitle from "../features/PageTitle";

function ErrorPage() {
  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`Not Found`} />
      <Result
        status="error"
        title="Not Found"
        subTitle="The page you are looking for does not exist."
        extra={[
          <Button
            key="retry"
            type="primary"
            onClick={() => window.history.back()}
          >
            Return to previous page
          </Button>,
        ]}
      />
    </div>
  );
}

export default ErrorPage;

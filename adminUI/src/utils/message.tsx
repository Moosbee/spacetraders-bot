import { useEffect } from "react";

import { message } from "antd";
import { MESSAGE_EVENT_NAME } from "./antdMessage";

const MessageAntD = () => {
  const [api, contextHolder] = message.useMessage();

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const bindEvent = (e: CustomEvent | any) => {
      const func = e?.detail?.type || "info";
      // eslint-disable-next-line no-unsafe-optional-chaining
      const { content, duration, onClose } = e.detail?.params;
      // @ts-expect-error because any
      api[func](content, duration, onClose);
    };

    window.addEventListener(MESSAGE_EVENT_NAME, bindEvent);

    return () => {
      window.removeEventListener(MESSAGE_EVENT_NAME, bindEvent);
    };
  }, [api]);

  return <>{contextHolder}</>;
};

export default MessageAntD;

import Form from "@rjsf/core";
import validator from "@rjsf/validator-ajv8";
import { useStore } from "@/store";
import { uiSchema } from "@/forms/pipeline-uiSchema";
import schema from "../../../schema/pipeline-config.schema.json" with { type: "json" };
import type { UiSchema } from "@rjsf/utils";

type JSONSchema7 = Record<string, unknown>;
const rjsfSchema = schema as JSONSchema7;

export default function ConfigForm() {
  const config = useStore((s) => s.config);
  const setConfig = useStore((s) => s.setConfig);

  return (
    <div className="space-y-4">
      <h2 className="text-sm font-semibold text-foreground tracking-wide uppercase">
        Pipeline Config
      </h2>
      <Form
        schema={rjsfSchema}
        uiSchema={uiSchema as UiSchema}
        validator={validator}
        formData={config}
        onChange={(e) => setConfig(e.formData as typeof config)}
        onSubmit={() => {}}
        onError={() => {}}
        liveValidate={false}
        showErrorList={false}
      >
        {/* no default submit button — we use external process trigger */}
        <div />
      </Form>
    </div>
  );
}

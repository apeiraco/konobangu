import { Code2, Play, Settings, Type } from "lucide-react";
import { type FC, useCallback, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Cron } from "./cron.jsx";
import { CronBuilder } from "./cron-builder.jsx";
import { CronDisplay } from "./cron-display.jsx";
import { CronInput } from "./cron-input.jsx";

const CronExample: FC = () => {
  const [inputValue, setInputValue] = useState("0 30 9 * * 1-5");
  const [builderValue, setBuilderValue] = useState("0 0 */6 * * *");
  const [fullValue, setFullValue] = useState("0 */15 * * * *");
  const [displayValue] = useState("0 0 0 * * 0");

  const examples = [
    {
      label: "Every minute",
      expression: "0 * * * * *",
      description: "Runs at the start of every minute",
    },
    {
      label: "Every 5 minutes",
      expression: "0 */5 * * * *",
      description: "Runs every 5 minutes",
    },
    {
      label: "Every hour",
      expression: "0 0 * * * *",
      description: "Runs at the start of every hour",
    },
    {
      label: "Daily at 9 AM",
      expression: "0 0 9 * * *",
      description: "Runs every day at 9:00 AM",
    },
    {
      label: "Weekdays at 9:30 AM",
      expression: "0 30 9 * * 1-5",
      description: "Runs Monday through Friday at 9:30 AM",
    },
    {
      label: "Every Sunday",
      expression: "0 0 0 * * 0",
      description: "Runs every Sunday at midnight",
    },
    {
      label: "First day of month",
      expression: "0 0 0 1 * *",
      description: "Runs on the 1st day of every month",
    },
    {
      label: "Every quarter",
      expression: "0 0 0 1 */3 *",
      description: "Runs on the 1st day of every quarter",
    },
  ];

  const handleCopyExample = useCallback(async (expression: string) => {
    try {
      await navigator.clipboard.writeText(expression);
    } catch (error) {
      console.warn("Failed to copy to clipboard:", error);
    }
  }, []);

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="space-y-2">
        <h1 className="font-bold text-3xl">Cron Expression Components</h1>
        <p className="text-lg text-muted-foreground">
          A comprehensive set of components for creating and managing cron
          expressions.
        </p>
      </div>

      {/* Quick Examples */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Code2 className="h-5 w-5" />
            Common Examples
          </CardTitle>
          <CardDescription>
            Click any example to copy the expression to your clipboard
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {examples.map((example, index) => (
              <Button
                type="button"
                key={index}
                variant="outline"
                className="h-auto flex-col items-start p-4 text-left"
                onClick={() => handleCopyExample(example.expression)}
              >
                <div className="w-full space-y-2">
                  <div className="font-medium text-sm">{example.label}</div>
                  <Badge variant="secondary" className="font-mono text-xs">
                    {example.expression}
                  </Badge>
                  <div className="text-muted-foreground text-xs">
                    {example.description}
                  </div>
                </div>
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      <Separator />

      {/* Component Examples */}
      <div className="space-y-8">
        <div className="space-y-2">
          <h2 className="font-semibold text-2xl">Component Examples</h2>
          <p className="text-muted-foreground">
            Interactive examples showing different ways to use the cron
            components.
          </p>
        </div>

        <Tabs defaultValue="full" className="space-y-6">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="full">Complete</TabsTrigger>
            <TabsTrigger value="input">Input Only</TabsTrigger>
            <TabsTrigger value="builder">Builder Only</TabsTrigger>
            <TabsTrigger value="display">Display Only</TabsTrigger>
          </TabsList>

          <TabsContent value="full" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Settings className="h-5 w-5" />
                  Complete Cron Component
                </CardTitle>
                <CardDescription>
                  Full-featured component with both input and visual builder
                  modes, validation, preview, and help documentation.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <Cron
                    value={fullValue}
                    onChange={setFullValue}
                    mode="both"
                    showPreview={true}
                    showDescription={true}
                    timezone="UTC"
                  />

                  <div className="rounded bg-muted p-4">
                    <h4 className="mb-2 font-medium text-sm">Current Value:</h4>
                    <Badge variant="outline" className="font-mono">
                      {fullValue || "No expression set"}
                    </Badge>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="input" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Type className="h-5 w-5" />
                  Text Input Component
                </CardTitle>
                <CardDescription>
                  Simple text input with validation, help text, and real-time
                  feedback.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <CronInput
                    value={inputValue}
                    onChange={setInputValue}
                    placeholder="Enter cron expression..."
                  />

                  <div className="rounded bg-muted p-4">
                    <h4 className="mb-2 font-medium text-sm">Current Value:</h4>
                    <Badge variant="outline" className="font-mono">
                      {inputValue || "No expression set"}
                    </Badge>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Input-Only Mode</CardTitle>
                <CardDescription>
                  Using the main Cron component in input-only mode with preview.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Cron
                  value={inputValue}
                  onChange={setInputValue}
                  mode="input"
                  showPreview={true}
                />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="builder" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Settings className="h-5 w-5" />
                  Visual Builder Component
                </CardTitle>
                <CardDescription>
                  Visual interface for building cron expressions with presets
                  and field editors.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <CronBuilder
                    value={builderValue}
                    onChange={setBuilderValue}
                    showPreview={true}
                  />

                  <div className="rounded bg-muted p-4">
                    <h4 className="mb-2 font-medium text-sm">Current Value:</h4>
                    <Badge variant="outline" className="font-mono">
                      {builderValue || "No expression set"}
                    </Badge>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Builder-Only Mode</CardTitle>
                <CardDescription>
                  Using the main Cron component in builder-only mode.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Cron
                  value={builderValue}
                  onChange={setBuilderValue}
                  mode="builder"
                  showPreview={false}
                />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="display" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Play className="h-5 w-5" />
                  Display Component
                </CardTitle>
                <CardDescription>
                  Read-only component that shows cron expression details,
                  description, and next run times.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <CronDisplay
                    expression={displayValue}
                    showNextRuns={true}
                    showDescription={true}
                    nextRunsCount={5}
                    timezone="UTC"
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Multiple Timezone Display</CardTitle>
                <CardDescription>
                  Same expression displayed in different timezones.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-4 lg:grid-cols-2">
                  <div>
                    <h4 className="mb-2 font-medium text-sm">UTC</h4>
                    <CronDisplay
                      expression="0 0 12 * * *"
                      showNextRuns={true}
                      nextRunsCount={3}
                      timezone="UTC"
                    />
                  </div>
                  <div>
                    <h4 className="mb-2 font-medium text-sm">
                      America/New_York
                    </h4>
                    <CronDisplay
                      expression="0 0 12 * * *"
                      showNextRuns={true}
                      nextRunsCount={3}
                      timezone="America/New_York"
                    />
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>

      {/* Usage Examples */}
      <Card>
        <CardHeader>
          <CardTitle>Usage Examples</CardTitle>
          <CardDescription>
            Code examples showing how to integrate these components into your
            application.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            <div>
              <h4 className="mb-2 font-medium text-sm">Basic Usage</h4>
              <div className="rounded bg-muted p-4 font-mono text-sm">
                <pre>{`import { Cron } from '@/components/cron';

function MyComponent() {
  const [cronExpression, setCronExpression] = useState('0 0 * * * *');

  return (
    <Cron
      value={cronExpression}
      onChange={setCronExpression}
      mode="both"
      showPreview={true}
    />
  );
}`}</pre>
              </div>
            </div>

            <div>
              <h4 className="mb-2 font-medium text-sm">
                Input Only with Validation
              </h4>
              <div className="rounded bg-muted p-4 font-mono text-sm">
                <pre>{`import { CronInput } from '@/components/cron';

function ScheduleForm() {
  const [expression, setExpression] = useState('');
  const [isValid, setIsValid] = useState(false);

  return (
    <CronInput
      value={expression}
      onChange={setExpression}
      onValidate={setIsValid}
      placeholder="0 0 * * * *"
    />
  );
}`}</pre>
              </div>
            </div>

            <div>
              <h4 className="mb-2 font-medium text-sm">
                Display Schedule Information
              </h4>
              <div className="rounded bg-muted p-4 font-mono text-sm">
                <pre>{`import { CronDisplay } from '@/components/cron';

function SchedulePreview({ schedule }) {
  return (
    <CronDisplay
      expression={schedule.cronExpression}
      showNextRuns={true}
      showDescription={true}
      timezone={schedule.timezone}
    />
  );
}`}</pre>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export { CronExample };

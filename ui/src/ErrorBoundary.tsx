import { Component, type ErrorInfo, type ReactNode } from "react";
import { Card } from "primereact/card";
import { Button } from "primereact/button";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Uncaught error:", error, info);
  }

  render() {
    if (!this.state.hasError) {
      return this.props.children;
    }

    const header = (
      <img
        src="/images/platform-team.jpg"
        alt="Confused cats"
        className="w-full object-cover"
        style={{ maxHeight: 320 }}
      />
    );

    return (
      <div className="flex min-h-screen items-center justify-center bg-[#12071f] p-6">
        <Card
          header={header}
          className="w-full max-w-lg shadow-2xl overflow-hidden"
        >
          <div className="flex flex-col items-center gap-4 text-center py-2">
            <h1 className="text-2xl font-bold text-purple-100">
              Something went unexpectedly wrong
            </h1>
            <p className="text-purple-400 leading-relaxed">
              Looks like our app let the cat out of the bag — and now it's
              knocked everything off the table.{" "}
              <span className="text-purple-500 font-medium">
                Don't worry, we're on it (once we find the cat).
              </span>
            </p>
            <Button
              label="Try again"
              icon="pi pi-refresh"
              onClick={() => {
                this.setState({ hasError: false });
                window.location.href = "/";
              }}
              className="mt-2"
            />
          </div>
        </Card>
      </div>
    );
  }
}

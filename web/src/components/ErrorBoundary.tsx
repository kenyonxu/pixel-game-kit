import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
  label?: string;
}
interface State {
  error: Error | null;
}

/** Catches render errors in children so a single component crash (e.g. an RJSF
 *  widget/schema bug) shows a fallback instead of blanking the whole page. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(`[${this.props.label ?? "ErrorBoundary"}]`, error, info);
  }

  reset = () => this.setState({ error: null });

  render() {
    if (this.state.error) {
      return (
        <div className="text-xs text-destructive bg-destructive/10 border border-destructive/30 rounded p-3 space-y-2">
          <p className="font-medium">
            {this.props.label ?? "Component"} failed to render:
          </p>
          <p className="font-mono break-all">{this.state.error.message}</p>
          <button
            onClick={this.reset}
            className="text-primary underline-offset-4 hover:underline cursor-pointer"
          >
            Retry
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

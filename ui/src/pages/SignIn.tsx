import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card } from "primereact/card";
import { InputText } from "primereact/inputtext";
import { Password } from "primereact/password";
import { Button } from "primereact/button";
import { Message } from "primereact/message";
import { FloatLabel } from "primereact/floatlabel";
import { signInHandler } from "../api/sdk.gen";
import type { Cat } from "../api/types.gen";
import { useAuthStore } from "../store/authStore";

interface SignInResults {
  cat: Cat;
  session_id: string;
}

interface SignInResponseWithResults {
  status: string;
  message: string;
  results?: SignInResults;
}

export function SignIn() {
  const navigate = useNavigate();
  const setAuth = useAuthStore((s) => s.setAuth);

  const [name, setName] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const { data, error: apiError } = await signInHandler({
        body: { name, password },
      });

      if (apiError || !data) {
        setError("Sign in failed. Please try again.");
        return;
      }

      // The generated type only models the OpenAPI-declared shape, but the
      // actual server response includes `results: { cat, session_id }`.
      const rich = data as unknown as SignInResponseWithResults;

      if (rich.status !== "OK" || !rich.results) {
        setError(rich.message ?? "Sign in failed. Please try again.");
        return;
      }

      setAuth(rich.results.cat, rich.results.session_id);
      navigate("/matches");
    } catch {
      setError("An unexpected error occurred. Please try again.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-slate-900">
      <Card className="w-full max-w-md shadow-lg">
        <div className="mb-6 text-center">
          <h1 className="text-3xl font-bold text-slate-100">🐱 Meow Mingle</h1>
          <p className="mt-1 text-slate-400">Sign in to your account</p>
        </div>

        <form onSubmit={handleSubmit} className="flex flex-col gap-6">
          <FloatLabel>
            <InputText
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full"
              required
              autoComplete="username"
              maxLength={36}
            />
            <label htmlFor="name">Name</label>
          </FloatLabel>

          <FloatLabel>
            <Password
              inputId="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full"
              inputClassName="w-full"
              toggleMask
              required
              autoComplete="current-password"
              feedback={false}
              maxLength={150}
            />
            <label htmlFor="password">Password</label>
          </FloatLabel>

          {error && (
            <Message severity="error" text={error} className="w-full" />
          )}

          <Button
            type="submit"
            label="Sign In"
            icon="pi pi-sign-in"
            loading={loading}
            className="w-full"
          />

          <p className="text-center text-sm text-slate-400">
            Don't have an account?{" "}
            <a href="/signup" className="text-purple-400 hover:underline">
              Sign up
            </a>
          </p>
        </form>
      </Card>
    </div>
  );
}

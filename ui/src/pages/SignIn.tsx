import { useState, useRef } from "react";
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
import { AuthLayout } from "../components/AuthLayout";

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
  const [shaking, setShaking] = useState(false);
  const shakeTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);

  function triggerShake() {
    if (shakeTimeout.current) clearTimeout(shakeTimeout.current);
    setShaking(true);
    shakeTimeout.current = setTimeout(() => setShaking(false), 500);
  }

  async function handleSubmit(e: React.SubmitEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const { data, error: apiError } = await signInHandler({
        body: { name, password },
      });

      if (apiError || !data) {
        setError("Sign in failed. Please try again.");
        triggerShake();
        return;
      }

      // The generated type only models the OpenAPI-declared shape, but the
      // actual server response includes `results: { cat, session_id }`.
      const rich = data as unknown as SignInResponseWithResults;

      if (rich.status !== "OK" || !rich.results) {
        setError(rich.message ?? "Sign in failed. Please try again.");
        triggerShake();
        return;
      }

      setAuth(rich.results.cat, rich.results.session_id);
      navigate("/matches");
    } catch {
      setError("An unexpected error occurred. Please try again.");
      triggerShake();
    } finally {
      setLoading(false);
    }
  }

  return (
    <AuthLayout>
      <style>{`
        @keyframes shake {
          0%   { transform: translateX(0); }
          15%  { transform: translateX(-8px); }
          30%  { transform: translateX(8px); }
          45%  { transform: translateX(-6px); }
          60%  { transform: translateX(6px); }
          75%  { transform: translateX(-3px); }
          90%  { transform: translateX(3px); }
          100% { transform: translateX(0); }
        }
        .shake {
          animation: shake 0.5s ease-in-out;
        }
      `}</style>
      <Card className={`w-full max-w-md shadow-lg${shaking ? " shake" : ""}`}>
        <div className="mb-6 text-center">
          <h1 className="text-3xl font-bold text-purple-100">🐱 Meow Mingle</h1>
          <p className="mt-1 text-purple-400">Sign in to your account</p>
        </div>

        <form onSubmit={handleSubmit} className="flex flex-col gap-6">
          <FloatLabel className="w-full">
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

          <FloatLabel className="w-full">
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
              pt={{ root: { className: "w-full" } }}
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

          <p className="text-center text-sm text-purple-400">
            Don't have an account?{" "}
            <a href="/signup" className="text-purple-500 hover:underline">
              Sign up
            </a>
          </p>
        </form>
      </Card>
    </AuthLayout>
  );
}

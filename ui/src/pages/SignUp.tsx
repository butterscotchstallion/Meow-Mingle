import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card } from "primereact/card";
import { InputText } from "primereact/inputtext";
import { Password } from "primereact/password";
import { Button } from "primereact/button";
import { Message } from "primereact/message";
import { FloatLabel } from "primereact/floatlabel";
import { signUpHandler } from "../api/sdk.gen";

const MAINE_COON_BREED_ID = "910ee31d-1fb6-428c-8b84-418cb8e55f20";

export function SignUp() {
  const navigate = useNavigate();

  const [name, setName] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);

    if (password !== confirmPassword) {
      setError("Passwords do not match.");
      return;
    }

    setLoading(true);
    try {
      const { data, error: apiError } = await signUpHandler({
        body: {
          cat: {
            name,
            password,
            breed_id: MAINE_COON_BREED_ID,
          },
        },
      });

      if (apiError || !data?.results) {
        setError(data?.message ?? "Sign up failed. Please try again.");
        return;
      }

      navigate("/");
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
          <p className="mt-1 text-slate-400">Create your account</p>
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
              autoComplete="new-password"
              feedback={false}
            />
            <label htmlFor="password">Password</label>
          </FloatLabel>

          <FloatLabel>
            <Password
              inputId="confirmPassword"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              className="w-full"
              inputClassName="w-full"
              toggleMask
              required
              autoComplete="new-password"
              feedback={false}
            />
            <label htmlFor="confirmPassword">Confirm Password</label>
          </FloatLabel>

          {error && (
            <Message severity="error" text={error} className="w-full" />
          )}

          <Button
            type="submit"
            label="Create Account"
            icon="pi pi-user-plus"
            loading={loading}
            className="w-full"
          />

          <p className="text-center text-sm text-slate-400">
            Already have an account?{" "}
            <a href="/signin" className="text-purple-400 hover:underline">
              Sign in
            </a>
          </p>
        </form>
      </Card>
    </div>
  );
}

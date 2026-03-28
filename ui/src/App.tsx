import { Navigate, Route, Routes } from "react-router-dom";
import { SignUp } from "./pages/SignUp";
import { SignIn } from "./pages/SignIn";
import { Matches } from "./pages/Matches";
import { EditProfile } from "./pages/EditProfile";
import { useSessionSync } from "./hooks/useSessionSync";

export function App() {
  useSessionSync();

  return (
    <Routes>
      <Route path="/" element={<Navigate to="/matches" replace />} />
      <Route path="/signup" element={<SignUp />} />
      <Route path="/signin" element={<SignIn />} />
      <Route path="/matches" element={<Matches />} />
      <Route path="/edit-profile" element={<EditProfile />} />
    </Routes>
  );
}

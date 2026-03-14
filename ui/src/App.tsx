import { Route, Routes } from "react-router-dom";
import { Home } from "./pages/Home";
import { SignUp } from "./pages/SignUp";
import { SignIn } from "./pages/SignIn";
import { Matches } from "./pages/Matches";

export function App() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/signup" element={<SignUp />} />
      <Route path="/signin" element={<SignIn />} />
      <Route path="/matches" element={<Matches />} />
    </Routes>
  );
}

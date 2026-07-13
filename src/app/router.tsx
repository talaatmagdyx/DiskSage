import { Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "../components/layout/AppShell";
import { DashboardPage } from "../pages/DashboardPage";
import { DuplicatesPage } from "../pages/DuplicatesPage";
import { FindingsPage } from "../pages/FindingsPage";
import { ScanPage } from "../pages/ScanPage";
import { SettingsPage } from "../pages/SettingsPage";
import { HistoryPage } from "../pages/HistoryPage";

export function AppRouter() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<DashboardPage />} />
        <Route path="scan" element={<ScanPage />} />
        <Route path="cleanup" element={<FindingsPage />} />
        <Route path="duplicates" element={<DuplicatesPage />} />
        <Route path="history" element={<HistoryPage />} />
        <Route path="settings" element={<SettingsPage />} />
      </Route>
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

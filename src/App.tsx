import { AppLayout } from "@/components/layout/AppLayout";
import { AppShellProvider } from "@/app/AppShellContext";
import { ThemeProvider } from "@/theme/ThemeProvider";

export default function App() {
  return (
    <ThemeProvider>
      <AppShellProvider>
        <AppLayout />
      </AppShellProvider>
    </ThemeProvider>
  );
}

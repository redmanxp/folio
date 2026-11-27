import { useAppShell } from "@/app/AppShellContext";
import { Header } from "@/components/layout/Header";
import { Sidebar } from "@/components/layout/Sidebar";
import { DashboardPage } from "@/pages/DashboardPage";
import { DocumentsPage } from "@/pages/DocumentsPage";
import { InvoicesPage } from "@/pages/InvoicesPage";
import { PaymentsPage } from "@/pages/PaymentsPage";
import { ProvidersPage } from "@/pages/ProvidersPage";
import { ReportsPage } from "@/pages/ReportsPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { AboutPage } from "@/pages/institutional/AboutPage";
import { ContactPage } from "@/pages/institutional/ContactPage";
import { HelpPage } from "@/pages/institutional/HelpPage";
import { PrivacyPage } from "@/pages/institutional/PrivacyPage";
import { SupportPage } from "@/pages/institutional/SupportPage";

export function AppLayout() {
  const { section } = useAppShell();

  const content = (() => {
    switch (section) {
      case "home":
        return <DashboardPage />;
      case "invoices":
        return <InvoicesPage />;
      case "providers":
        return <ProvidersPage />;
      case "payments":
        return <PaymentsPage />;
      case "documents":
        return <DocumentsPage />;
      case "reports":
        return <ReportsPage />;
      case "settings":
        return <SettingsPage />;
      case "about":
        return <AboutPage />;
      case "support":
        return <SupportPage />;
      case "help":
        return <HelpPage />;
      case "contact":
        return <ContactPage />;
      case "privacy":
        return <PrivacyPage />;
      default:
        return <DashboardPage />;
    }
  })();

  return (
    <div className="flex h-full min-h-full bg-background">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <Header />
        <main className="flex-1 overflow-auto p-6">{content}</main>
      </div>
    </div>
  );
}

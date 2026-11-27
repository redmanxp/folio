import { useAppShell } from "@/app/AppShellContext";
import {
  FeatureGrid,
  InfoCard,
  InstitutionalShell,
} from "@/components/institutional/InstitutionalShell";
import { Button } from "@/components/ui/button";

export function PrivacyPage() {
  const { setSection } = useAppShell();

  return (
    <InstitutionalShell>
      <InfoCard
        title="Privacidad de Folio"
        description="Folio está diseñado para funcionar localmente en tu computadora."
      >
        <FeatureGrid
          items={[
            {
              title: "Tus documentos",
              body: "Facturas, proveedores, pagos y demás datos se guardan en tu dispositivo.",
            },
            {
              title: "Sin cuentas",
              body: "No necesitás registrarte ni dar datos personales para usar Folio.",
            },
            {
              title: "Sin telemetría",
              body: "No hay seguimiento de uso obligatorio dentro de la aplicación.",
            },
            {
              title: "Sin publicidad",
              body: "Folio no incorpora publicidad en la app.",
            },
          ]}
        />
      </InfoCard>

      <InfoCard title="Internet">
        <p>
          Las funciones principales no requieren conexión. Algunas acciones externas —como abrir
          enlaces de ayuda, GitHub, donaciones o consultar actualizaciones— pueden necesitar
          Internet y se hacen solo cuando vos las iniciás.
        </p>
      </InfoCard>

      <InfoCard title="Copias de seguridad">
        <p>
          Folio permite crear y restaurar backups. Sos responsable de guardar esos archivos en
          ubicaciones que consideres seguras.
        </p>
      </InfoCard>

      <InfoCard title="Transparencia">
        <p>
          El proyecto busca un desarrollo abierto, con código y documentación públicos a medida que
          se publiquen.
        </p>
        <p className="text-xs">
          Esta política se actualizará si cambian mecanismos de actualización o servicios externos.
        </p>
        <Button type="button" variant="outline" onClick={() => setSection("contact")}>
          Contacto
        </Button>
      </InfoCard>
    </InstitutionalShell>
  );
}

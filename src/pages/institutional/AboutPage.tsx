import { useEffect, useState } from "react";
import { ExternalLink, HeartHandshake, RefreshCw } from "lucide-react";
import { api } from "@/api";
import { useAppShell } from "@/app/AppShellContext";
import { BrandLogo } from "@/components/brand/BrandLogo";
import {
  FeatureGrid,
  InfoCard,
  InstitutionalShell,
  SoonButton,
} from "@/components/institutional/InstitutionalShell";
import { Button } from "@/components/ui/button";
import { FOLIO_META, hasLink } from "@/content/folioMeta";
import { openFolioLink } from "@/lib/openExternal";

export function AboutPage() {
  const { setSection } = useAppShell();
  const [version, setVersion] = useState<string>(FOLIO_META.versionLabel);

  useEffect(() => {
    void api.getAppInfo().then((info) => setVersion(info.version)).catch(() => undefined);
  }, []);

  return (
    <InstitutionalShell>
      <InfoCard title={FOLIO_META.name} description={FOLIO_META.tagline}>
        <div className="flex flex-col items-start gap-4 sm:flex-row sm:items-center">
          <BrandLogo variant="full" className="h-14 w-auto" />
          <p className="text-xs text-muted-foreground">
            Gratis · {FOLIO_META.platform} · v{version}
          </p>
        </div>
      </InfoCard>

      <InfoCard title="¿Qué es Folio?">
        <p>
          Folio es una aplicación gratuita diseñada para ayudarte a organizar facturas, documentos,
          proveedores, pagos y vencimientos, de forma sencilla y desde tu propia computadora.
        </p>
      </InfoCard>

      <InfoCard title="Nuestra filosofía">
        <p>
          Creemos que una herramienta útil no debería obligarte a pagar una suscripción, registrarte
          o subir tus documentos a Internet.
        </p>
        <p>
          Folio está pensado para que puedas organizar tu información, mantener el control de tus
          archivos y utilizarlo sin depender de servicios externos.
        </p>
        <FeatureGrid
          items={[
            {
              title: "Privacidad",
              body: "Tus documentos permanecen en tu computadora.",
            },
            {
              title: "Gratis",
              body: "Sin suscripciones ni funciones básicas bloqueadas.",
            },
            {
              title: "Offline",
              body: "Sin necesidad de conexión para el uso habitual.",
            },
            {
              title: "Transparente",
              body: "Código público y desarrollo abierto.",
            },
          ]}
        />
      </InfoCard>

      <InfoCard title="Actualizaciones">
        <p>
          Versión instalada: <span className="font-medium text-foreground">Folio {version}</span>
        </p>
        <p>Las actualizaciones serán avisadas para que vos decidas cuándo descargar.</p>
        {hasLink("downloads") ? (
          <Button type="button" variant="outline" onClick={() => void openFolioLink("downloads")}>
            <RefreshCw className="h-4 w-4" />
            Buscar actualizaciones
          </Button>
        ) : (
          <SoonButton label="Buscar actualizaciones" />
        )}
      </InfoCard>

      <InfoCard title="Enlaces">
        <div className="flex flex-wrap gap-2">
          {hasLink("website") ? (
            <Button type="button" variant="outline" onClick={() => void openFolioLink("website")}>
              <ExternalLink className="h-4 w-4" />
              Sitio oficial
            </Button>
          ) : (
            <SoonButton label="Sitio oficial" />
          )}
          {hasLink("github") ? (
            <Button type="button" variant="outline" onClick={() => void openFolioLink("github")}>
              <ExternalLink className="h-4 w-4" />
              GitHub
            </Button>
          ) : (
            <SoonButton label="GitHub" />
          )}
          <Button type="button" variant="outline" onClick={() => setSection("contact")}>
            Contacto
          </Button>
          <Button type="button" variant="outline" onClick={() => setSection("privacy")}>
            Privacidad
          </Button>
          <Button type="button" variant="outline" onClick={() => setSection("support")}>
            <HeartHandshake className="h-4 w-4" />
            Apoyar el proyecto
          </Button>
        </div>
        <p className="text-xs">Licencia: {FOLIO_META.license} (detalle pendiente de publicación).</p>
      </InfoCard>
    </InstitutionalShell>
  );
}

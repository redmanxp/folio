import {
  Building2,
  FileUp,
  HardDrive,
  Shield,
  Users,
  Wallet,
} from "lucide-react";
import { useAppShell } from "@/app/AppShellContext";
import {
  InfoCard,
  InstitutionalShell,
} from "@/components/institutional/InstitutionalShell";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const TOPICS = [
  {
    title: "Agregar documentos",
    body: "En Facturas usá «Elegir archivos» para importar PDF o imágenes. Folio copia el archivo, intenta leer QR/texto y asocia proveedor por CUIT.",
    icon: FileUp,
  },
  {
    title: "Entidades y empresas",
    body: "Cada entidad es un espacio de trabajo (personal o empresa). Cambiá la activa desde la barra lateral; los datos no se mezclan entre entidades.",
    icon: Building2,
  },
  {
    title: "Proveedores",
    body: "Se crean al importar por CUIT o manualmente. Desde Facturas podés abrir la ficha; editá alias, CBU, correo, teléfono y notas.",
    icon: Users,
  },
  {
    title: "Pagos y vencimientos",
    body: "En modo gestión completa, Pagos muestra pendientes y fechas. El vencimiento se completa cuando el PDF o el QR aportan esa fecha.",
    icon: Wallet,
  },
  {
    title: "Copias de seguridad",
    body: "En Configuración podés crear y restaurar backups ZIP de la base y los archivos. Guardá los respaldos en un lugar seguro.",
    icon: HardDrive,
  },
  {
    title: "Privacidad y seguridad",
    body: "Folio guarda todo en tu PC. No crea cuenta ni envía tus facturas a Internet. Detalle en la sección Privacidad.",
    icon: Shield,
  },
] as const;

export function HelpPage() {
  const { setSection } = useAppShell();

  return (
    <InstitutionalShell>
      <InfoCard
        title="¿Qué necesitás saber?"
        description="Guía rápida incluida en la app. No requiere Internet."
      >
        <div className="flex flex-wrap gap-2">
          <Button type="button" variant="outline" onClick={() => setSection("contact")}>
            Contacto y sugerencias
          </Button>
          <Button type="button" variant="outline" onClick={() => setSection("privacy")}>
            Privacidad
          </Button>
          <Button type="button" variant="outline" onClick={() => setSection("about")}>
            Acerca de Folio
          </Button>
        </div>
      </InfoCard>

      <div className="grid gap-3">
        {TOPICS.map((topic) => {
          const Icon = topic.icon;
          return (
            <Card key={topic.title}>
              <CardHeader>
                <div className="flex items-center gap-2">
                  <Icon className="h-4 w-4 text-brand-blue" />
                  <CardTitle>{topic.title}</CardTitle>
                </div>
              </CardHeader>
              <CardContent className="text-sm leading-relaxed text-muted-foreground">
                {topic.body}
              </CardContent>
            </Card>
          );
        })}
      </div>
    </InstitutionalShell>
  );
}

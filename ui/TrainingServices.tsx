import { useContext, useEffect, useState } from "react";
import { api } from "./api";
import { SearchSelect } from "./SearchSelect";
import { TrainingServicePreview } from "./TrainingServicePreview";
import { ConditionDetails, ConditionQueryRevision } from "./ConditionDetails";
import { useI18n } from "./i18n";
import type {
  AcquisitionSource,
  Catalog,
  MapFocus,
  QueryTarget,
  Snapshot,
} from "./types";

type Check = AcquisitionSource["conditions"][number];
export type TrainingService = {
  kind: string;
  minimum_level: number;
  choices: {
    menu_index: number;
    name: string;
    item: number;
    quantity: number;
    stat: number | null;
    mask: number;
    credit: Check | null;
    item_requirement: Check;
    payment: {
      item: number;
      quantity: number;
      before_stat_selection: boolean;
      result_checked: boolean;
    };
  }[];
  locations: {
    map_id: string;
    map_name: string;
    x: number;
    y: number;
    visibility: Check[];
  }[];
  text: string[];
  conditions: Check[];
  evidence: { root: number };
  menus: {
    stage: string;
    single_stat_only: boolean;
    cancel_with_b: boolean;
    payment_precedes_menu: boolean;
    command: number;
  }[];
  selection: {
    scope: string;
    cancel_with_b: boolean;
    rejects_fainted_during_selection: boolean;
    rejects_egg_during_selection: boolean;
  };
};
type Report = {
  rom_md5: string;
  services: TrainingService[];
  partial: boolean;
};

export function TrainingServices({
  catalog,
  save,
  onMap,
  onTarget,
  onError,
}: {
  catalog: Catalog;
  save: Snapshot | null;
  onMap: (id: string, focus?: MapFocus) => void;
  onTarget: (target: QueryTarget) => void;
  onError: (error: unknown) => void;
}) {
  const revision = useContext(ConditionQueryRevision);
  const md5 = catalog.profile.md5;
  const [state, setState] = useState<{
    report: Report;
    revision: number;
  } | null>(null);
  useEffect(() => {
    let cancelled = false;
    setState(null);
    api<Report>("training_services", { expected_rom_md5: md5 })
      .then((report) => {
        if (!cancelled && report.rom_md5 === md5)
          setState({ report, revision });
      })
      .catch((error) => {
        if (!cancelled) onError(error);
      });
    return () => {
      cancelled = true;
    };
  }, [md5, revision]);
  const services =
    state?.revision === revision && state.report.rom_md5 === md5
      ? state.report.services
      : [];
  return services.map((service, index) => (
    <ServiceCard
      key={index}
      service={service}
      catalog={catalog}
      save={save}
      onError={onError}
      onMap={onMap}
      onTarget={onTarget}
    />
  ));
}
function ServiceCard({
  service,
  catalog,
  save,
  onMap,
  onTarget,
  onError,
}: {
  service: TrainingService;
  catalog: Catalog;
  save: Snapshot | null;
  onMap: (id: string, focus?: MapFocus) => void;
  onTarget: (target: QueryTarget) => void;
  onError: (error: unknown) => void;
}) {
  const { t } = useI18n();
  const [selected, setSelected] = useState(0);
  const choice =
    service.choices.find((c) => c.menu_index === selected) ??
    service.choices[0];
  return (
    <article className="training-service">
      <h3>
        {t(
          service.kind === "base_iv_training"
            ? "trainingIvService"
            : "trainingCrownService",
        )}
      </h3>
      <p>
        {t("trainingCrownLevel")} {service.minimum_level}
      </p>
      {service.selection.scope === "party" && (
        <p className="small">
          {t("trainingServicePartyOnly")}
          {service.selection.cancel_with_b && (
            <span> {t("trainingServicePartyCancel")}</span>
          )}
        </p>
      )}
      {!service.selection.rejects_fainted_during_selection && (
        <p className="small muted">{t("trainingServiceFaintedSelection")}</p>
      )}
      <p className="small muted">
        {t(
          service.kind === "base_iv_training"
            ? "trainingIvEffect"
            : "trainingCrownEffect",
        )}
      </p>
      <p className="small muted">
        {t(
          service.kind === "base_iv_training"
            ? "trainingIvAccess"
            : "trainingCrownAccess",
        )}
      </p>
      <ConditionDetails
        checks={service.conditions}
        catalog={catalog}
        onMap={onMap}
        onTarget={onTarget}
      />
      {service.locations.map((location) => (
        <div key={`${location.map_id}:${location.x}:${location.y}`}>
          <button
            className="link-button"
            onClick={() =>
              onMap(location.map_id, { x: location.x, y: location.y })
            }
          >
            {location.map_name} · ({location.x}, {location.y}) ↗
          </button>
          <ConditionDetails
            checks={location.visibility}
            catalog={catalog}
            onMap={onMap}
            onTarget={onTarget}
          />
        </div>
      ))}
      <SearchSelect
        label={t("trainingCrownChoice")}
        value={choice?.menu_index ?? 0}
        options={service.choices.map((c) => ({
          value: c.menu_index,
          label: c.name,
        }))}
        onChange={(value) => setSelected(+value)}
      />
      {choice && (
        <section className="training-service-choice" key={choice.menu_index}>
          <strong>{choice.name}</strong>
          {" · "}
          {t("trainingServiceRequired")}{" "}
          <button
            className="link-button"
            onClick={() => onTarget({ kind: "item", id: choice.item })}
          >
            {catalog.items.find((item) => item.id === choice.item)?.name ??
              choice.item}{" "}
            × {choice.quantity} ↗
          </button>
          {service.kind === "base_iv_training" && (
            <p className="small">
              {t("trainingServicePayment")}{" "}
              <button
                className="link-button"
                onClick={() =>
                  onTarget({ kind: "item", id: choice.payment.item })
                }
              >
                {catalog.items.find((item) => item.id === choice.payment.item)
                  ?.name ?? choice.payment.item}{" "}
                × {choice.payment.quantity} ↗
              </button>
              {choice.payment.before_stat_selection && (
                <span> · {t("trainingServiceEarlyPayment")}</span>
              )}
            </p>
          )}
          {choice.payment.item !== choice.item && (
            <p className="small warning-text">
              {t("trainingServicePaymentMismatch")}
            </p>
          )}
          {service.menus
            .filter((menu) => !menu.single_stat_only || choice.stat !== null)
            .map((menu) => (
              <p className="small" key={menu.command}>
                {t(
                  menu.stage === "stat_choice"
                    ? "trainingServiceStatMenu"
                    : "trainingServiceMainMenu",
                )}
                {" · "}
                {t(
                  menu.cancel_with_b
                    ? "trainingServiceCanCancel"
                    : "trainingServiceCannotCancel",
                )}
                {menu.payment_precedes_menu && (
                  <span> · {t("trainingServicePaymentAlreadyAttempted")}</span>
                )}
              </p>
            ))}
          {choice.credit && (
            <p className="small">
              {t("trainingCrownCredit")} ·{" "}
              {choice.credit.actual ?? t("acqStatus_unknown")}
              {" · "}
              {t(
                choice.credit.satisfied === true
                  ? "planConditionYes"
                  : choice.credit.satisfied === false
                    ? "planConditionNo"
                    : "acqStatus_unknown",
              )}
            </p>
          )}
          <details>
            <summary>{t("trainingCrownConditions")}</summary>
            <ConditionDetails
              checks={
                choice.credit
                  ? [choice.credit, choice.item_requirement]
                  : [choice.item_requirement]
              }
              catalog={catalog}
              onMap={onMap}
              onTarget={onTarget}
            />
          </details>
        </section>
      )}
      {choice && (
        <TrainingServicePreview
          catalog={catalog}
          save={save}
          service={service}
          choice={choice}
          onError={onError}
        />
      )}
      <details>
        <summary>{t("trainingCrownDialogue")}</summary>
        {service.text.map((text, i) => (
          <p style={{ whiteSpace: "pre-wrap" }} key={i}>
            {text}
          </p>
        ))}
      </details>
      <details>
        <summary>{t("evidence")}</summary>
        <pre>{JSON.stringify(service, null, 2)}</pre>
      </details>
    </article>
  );
}

import { FormEvent, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AndroidImageManifest, DefaultImageSettings, InstalledImage } from "../types";

type Props = {
  images: InstalledImage[];
  onChanged: () => Promise<void>;
};

const defaultManifest: AndroidImageManifest = {
  id: "android-16-x86_64",
  name: "Android 16",
  androidVersion: "16",
  api: 36,
  architecture: "x86_64",
  imageType: "aosp",
  disk: "android16.qcow2",
  diskFormat: "qcow2",
  recommended: true,
  notes: "",
  sha256: "",
  gmsProvider: "none",
  certificationStatus: "Uncertified / unknown",
  playStorePackage: null,
  secureImage: false,
  verifiedBootState: "unknown",
  securityState: "virtualized / unknown",
  missingHardwareFeatures: ["hardware-backed attestation", "Pixel secure element"],
  bootKernel:null,
  bootInitrd:null,
  vendorDisk:null,
  rootCapable:false
};

export default function ImageManager({ images, onChanged }: Props) {
  const [manifest, setManifest] = useState(defaultManifest);
  const [sourceDisk, setSourceDisk] = useState("");
  const [downloadUrl, setDownloadUrl] = useState("");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);
  const [defaultImages,setDefaultImages]=useState<DefaultImageSettings>({url:"",sha256:null,rootDeveloperUrl:null,rootDeveloperSha256:null});
  const [gsi,setGsi]=useState({system:"",kernel:"",initrd:"",vendor:"",androidVersion:"16",rootCapable:false});

  useEffect(()=>{invoke<DefaultImageSettings>("get_default_image_settings").then(setDefaultImages).catch(()=>{});},[]);
  const updateImage = async (id: string, name: string) => {
    if (!window.confirm(`Update ${name} from its saved source URL? The downloaded image is verified before replacing the installed disk.`)) return;
    setBusy(true);
    try {
      const updated = await invoke<InstalledImage>("update_android_image", { id });
      setOutput(`Updated ${updated.manifest.name} at ${updated.diskPath}`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const repairImage = async (id: string, name: string) => {
    if (!window.confirm(`Re-download and repair ${name} from its saved source URL?`)) return;
    setBusy(true);
    try {
      const repaired = await invoke<InstalledImage>("repair_android_image", { id });
      setOutput(`Repaired ${repaired.manifest.name} at ${repaired.diskPath}`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const removeImage = async (id: string, name: string) => {
    if (!window.confirm(`Remove ${name} from NekoDroid? This deletes its copied image files.`)) return;
    setBusy(true);
    try {
      await invoke("remove_android_image", { id });
      setOutput(`Removed ${name}.`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const download = async () => {
    if (!downloadUrl.trim()) return;
    setBusy(true);
    try {
      const installed = await invoke<InstalledImage>("download_android_image", {
        manifest,
        url: downloadUrl.trim()
      });
      setOutput(`Downloaded and installed ${installed.manifest.name} at ${installed.diskPath}`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      const installed = await invoke<InstalledImage>("register_android_image", {
        manifest,
        sourceDisk
      });
      setOutput(`Registered ${installed.manifest.name} at ${installed.diskPath}`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Android Images</p><h3>Installed image registry</h3></div>
        <span className="pill">{images.length} registered</span>
      </div>

      <div className="image-manager-grid">
        <div>
          {images.length === 0 ? (
            <p className="muted">No Android image has been registered yet.</p>
          ) : (
            <div className="image-list">
              {images.map(image => (
                <article className="instance-card" key={image.manifest.id}>
                  <div className="instance-title">
                    <strong>{image.manifest.name}</strong>
                    <span className={image.valid ? "status status-running" : "status status-error"}>
                      {image.valid ? "valid" : "invalid"}
                    </span>
                  </div>
                  <p>Android {image.manifest.androidVersion} · API {image.manifest.api} · {image.manifest.architecture}</p>
                  <small>{image.manifest.imageType} · {image.manifest.diskFormat} · GMS: {image.manifest.gmsProvider}</small>
                  <small>Certification: {image.manifest.certificationStatus || "unknown"} · Verified boot: {image.manifest.verifiedBootState || "unknown"}</small>
                  <small>Security: {image.manifest.securityState || "unknown"}</small>
                  {image.manifest.missingHardwareFeatures?.length ? <small>Unavailable hardware features: {image.manifest.missingHardwareFeatures.join(", ")}</small> : null}
                  <small className="image-path">{image.diskPath}</small>
                  {image.validationError && <small>{image.validationError}</small>}
                  <div className="button-row">
                    {image.manifest.sourceUrl && (
                      <>
                        <button
                          className="ghost compact"
                          disabled={busy}
                          onClick={() => updateImage(image.manifest.id, image.manifest.name)}
                        >
                          Update from source
                        </button>
                        <button
                          className="ghost compact"
                          disabled={busy}
                          onClick={() => repairImage(image.manifest.id, image.manifest.name)}
                        >
                          Repair / Re-download
                        </button>
                      </>
                    )}
                    <button
                      className="danger compact"
                      disabled={busy}
                      onClick={() => removeImage(image.manifest.id, image.manifest.name)}
                    >
                      Remove image
                    </button>
                  </div>
                </article>
              ))}
            </div>
          )}
        </div>

        <form className="image-register-form" onSubmit={submit}>
          <h4>Default Android 16 images</h4>
          <label>Default image URL<input value={defaultImages.url} onChange={e=>setDefaultImages({...defaultImages,url:e.target.value})}/></label>
          <label>Default SHA-256<input value={defaultImages.sha256??""} onChange={e=>setDefaultImages({...defaultImages,sha256:e.target.value||null})}/></label>
          <label>Root developer image URL<input value={defaultImages.rootDeveloperUrl??""} onChange={e=>setDefaultImages({...defaultImages,rootDeveloperUrl:e.target.value||null})}/></label>
          <label>Root developer SHA-256<input value={defaultImages.rootDeveloperSha256??""} onChange={e=>setDefaultImages({...defaultImages,rootDeveloperSha256:e.target.value||null})}/></label>
          <div className="button-row">
            <button type="button" className="ghost compact" onClick={async()=>{try{setDefaultImages(await invoke<DefaultImageSettings>("save_default_image_settings",{settings:defaultImages}));setOutput("Default image sources saved.");}catch(error){setOutput(String(error));}}}>Save sources</button>
            <button type="button" className="primary compact" disabled={busy||!defaultImages.url} onClick={async()=>{try{setBusy(true);const image=await invoke<InstalledImage>("download_default_android_image",{root:false});setOutput(`Installed ${image.manifest.name}`);await onChanged();}catch(error){setOutput(String(error));}finally{setBusy(false);}}}>Download default Android 16</button>
            <button type="button" className="danger compact" disabled={busy||!defaultImages.rootDeveloperUrl} onClick={async()=>{try{setBusy(true);const image=await invoke<InstalledImage>("download_default_android_image",{root:true});setOutput(`Installed ${image.manifest.name}`);await onChanged();}catch(error){setOutput(String(error));}finally{setBusy(false);}}}>Download root developer image</button>
          </div>

          <h4>Custom GSI boot bundle</h4>
          <label>Android version<input value={gsi.androidVersion} onChange={e=>setGsi({...gsi,androidVersion:e.target.value})}/></label>
          <label>System GSI image<input value={gsi.system} onChange={e=>setGsi({...gsi,system:e.target.value})}/></label>
          <label>Kernel<input value={gsi.kernel} onChange={e=>setGsi({...gsi,kernel:e.target.value})}/></label>
          <label>Initrd / ramdisk<input value={gsi.initrd} onChange={e=>setGsi({...gsi,initrd:e.target.value})}/></label>
          <label>Vendor image (optional)<input value={gsi.vendor} onChange={e=>setGsi({...gsi,vendor:e.target.value})}/></label>
          <label className="checkbox-line"><input type="checkbox" checked={gsi.rootCapable} onChange={e=>setGsi({...gsi,rootCapable:e.target.checked})}/>Root-capable bundle</label>
          <button type="button" className="ghost compact" disabled={busy||!gsi.system||!gsi.kernel||!gsi.initrd} onClick={async()=>{try{setBusy(true);const image=await invoke<InstalledImage>("register_gsi_boot_bundle",{androidVersion:gsi.androidVersion,system:gsi.system,kernel:gsi.kernel,initrd:gsi.initrd,vendor:gsi.vendor||null,rootCapable:gsi.rootCapable});setOutput(`Registered GSI boot bundle ${image.manifest.name}`);await onChanged();}catch(error){setOutput(String(error));}finally{setBusy(false);}}}>Register GSI boot bundle</button>

          <h4>Register local image</h4>
          <label>Image ID<input value={manifest.id} onChange={e => setManifest({...manifest,id:e.target.value})} /></label>
          <label>Name<input value={manifest.name} onChange={e => setManifest({...manifest,name:e.target.value})} /></label>
          <div className="split-fields">
            <label>Android<input value={manifest.androidVersion} onChange={e => setManifest({...manifest,androidVersion:e.target.value})} /></label>
            <label>API<input type="number" value={manifest.api} onChange={e => setManifest({...manifest,api:Number(e.target.value)})} /></label>
          </div>
          <div className="split-fields">
            <label>Type
              <select value={manifest.imageType} onChange={e => setManifest({...manifest,imageType:e.target.value})}>
                <option value="aosp">AOSP</option>
                <option value="gapps">GApps</option>
                <option value="microg">microG</option>
                <option value="custom">Custom</option>
              </select>
            </label>
            <label>Format
              <select value={manifest.diskFormat} onChange={e => setManifest({...manifest,diskFormat:e.target.value as AndroidImageManifest["diskFormat"],disk:e.target.value === "qcow2" ? "android.qcow2" : "android.img"})}>
                <option value="qcow2">QCOW2</option>
                <option value="raw">RAW/IMG</option>
              </select>
            </label>
          </div>
          <label>GMS provider
            <select value={manifest.gmsProvider} onChange={e => setManifest({...manifest,gmsProvider:e.target.value as AndroidImageManifest["gmsProvider"]})}>
              <option value="none">None / AOSP</option>
              <option value="google-compatible">Legally obtained Google-compatible image</option>
              <option value="microg">microG profile</option>
              <option value="custom-gapps">Custom GApps profile</option>
            </select>
          </label>
          <label>Certification status<input value={manifest.certificationStatus} onChange={e => setManifest({...manifest,certificationStatus:e.target.value})} /></label>
          <label>Play Store package<input value={manifest.playStorePackage ?? ""} placeholder="com.android.vending" onChange={e => setManifest({...manifest,playStorePackage:e.target.value || null})} /></label>
          <label className="checkbox-line"><input type="checkbox" checked={manifest.secureImage} onChange={e => setManifest({...manifest,secureImage:e.target.checked})} />Generic custom secure-image profile</label>
          <label>Verified boot state<input value={manifest.verifiedBootState} onChange={e => setManifest({...manifest,verifiedBootState:e.target.value})} /></label>
          <label>Actual security state<input value={manifest.securityState} onChange={e => setManifest({...manifest,securityState:e.target.value})} /></label>
          <label>Missing hardware-backed features
            <textarea value={manifest.missingHardwareFeatures.join("\n")} onChange={e => setManifest({...manifest,missingHardwareFeatures:e.target.value.split(/\r?\n/).map(v => v.trim()).filter(Boolean)})} />
          </label>
          <div className="warning-box">Image metadata is descriptive only. NekoDroid does not claim genuine Pixel identity, hardware-backed attestation, Play Integrity certification, or secure-element features that the VM does not actually have.</div>
          <label>Stored disk filename<input value={manifest.disk} onChange={e => setManifest({...manifest,disk:e.target.value})} /></label>
          <label>Source disk path<input placeholder="C:\\Android\\android16.qcow2" value={sourceDisk} onChange={e => setSourceDisk(e.target.value)} /></label>
          <label>Image download URL
            <input
              placeholder="https://example.com/android16.qcow2"
              value={downloadUrl}
              onChange={e => setDownloadUrl(e.target.value)}
            />
          </label>
          <label>SHA-256 checksum (optional)
            <input
              placeholder="64 hexadecimal characters"
              value={manifest.sha256 ?? ""}
              onChange={e => setManifest({...manifest,sha256:e.target.value})}
            />
          </label>
          <div className="button-row">
            <button className="primary" disabled={busy || !sourceDisk}>{busy ? "Working..." : "Register Local Image"}</button>
            <button type="button" className="ghost" disabled={busy || !downloadUrl.trim()} onClick={download}>
              {busy ? "Working..." : "Download & Install"}
            </button>
          </div>
          {output && <pre className="inline-output">{output}</pre>}
        </form>
      </div>
    </section>
  );
}

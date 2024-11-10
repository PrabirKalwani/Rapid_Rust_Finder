import { ResultsContainer } from "@/components/ResultsContainer";
import {
  SidebarProvider,
  SidebarTrigger,
  SidebarInset,
} from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { Navbar } from "./Navbar";

export const ViewPage = ({
  results,
  loading,
  error,
  query,
  openFile,
  recent,
  getFileIcon,
  handleChange,
  selectedFile,
  setSelectedFile
}) => {
  return (
    <SidebarProvider>
      <AppSidebar query={query} handleChange={handleChange} />
      <SidebarInset>
        <main>
          <Navbar text={query != "" ? "Results" : "Recent Files"} path={selectedFile === null ? null : selectedFile.filePath}/>
          <ResultsContainer
            results={query != "" ? results : recent.getItems()}
            loading={loading}
            error={error}
            query={query}
            openFile={openFile}
            getFileIcon={getFileIcon}
            selectedFile={selectedFile}
            setSelectedFile={setSelectedFile}
          />
        </main>
      </SidebarInset>
    </SidebarProvider>
  );
};

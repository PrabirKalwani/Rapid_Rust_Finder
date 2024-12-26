import {
  FolderHeart,
  Folder,
  BookOpen,
  Bot,
  Command,
  Frame,
  GalleryVerticalEnd,
  Map,
  PieChart,
  Settings2,
  SquareTerminal,
} from "lucide-react"

import { SidebarGroupItems } from "@/components/SidebarGroupItems"
import { SearchForm } from "@/components/search-form"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"

// This is sample data.


export function AppSidebar({ ...props }) {
  const data = {
    quickAccess: props.keyFolders,
  }
  return (
    <Sidebar collapsible="offcanvas" {...props}>
      <SidebarHeader>
        <SearchForm query={props.query} handleChange={props.handleChange}/>
      </SidebarHeader>
      <SidebarContent className="">
        <SidebarGroupItems items={data.quickAccess} groupTitle={"Quick Access"} openFile={props.openFile}/>
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  )
}

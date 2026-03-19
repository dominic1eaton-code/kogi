import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ItembookWorkspaceComponent } from './itembook-workspace.component';

describe('ItembookWorkspaceComponent', () => {
  let component: ItembookWorkspaceComponent;
  let fixture: ComponentFixture<ItembookWorkspaceComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ItembookWorkspaceComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(ItembookWorkspaceComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
